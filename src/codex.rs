use std::{
    io::{BufRead, BufReader, Write},
    process::{Child, ChildStdin, Command, Stdio},
    sync::mpsc::{Receiver, RecvError, RecvTimeoutError, Sender, TryRecvError, channel},
    thread,
    time::Duration,
};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

use chrono::Local;
use serde_json::{Value, json};
use thiserror::Error;

const CREATE_NO_WINDOW: u32 = 0x0800_0000;
const COMMAND_CHECK_INTERVAL: Duration = Duration::from_millis(250);
const RECONNECT_DELAY: Duration = Duration::from_secs(10);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageSnapshot {
    pub used_percent: u32,
    pub window_duration_mins: Option<i64>,
    pub resets_at: Option<i64>,
    pub plan_type: Option<String>,
    pub credit_balance: Option<String>,
    pub has_credits: bool,
    pub unlimited_credits: bool,
    pub reset_credits: i64,
    pub limit_reached_type: Option<String>,
    pub updated_at: i64,
}

#[derive(Debug, Clone)]
pub enum WorkerUpdate {
    Querying,
    Snapshot(UsageSnapshot),
    Error { message: String, at: i64 },
}

#[derive(Debug, Clone, Copy)]
pub enum WorkerCommand {
    Stop,
}

#[derive(Debug, Error)]
pub enum CodexError {
    #[error("не удалось запустить codex: {0}")]
    Spawn(#[source] std::io::Error),
    #[error("ошибка обмена с codex app-server: {0}")]
    Io(#[from] std::io::Error),
    #[error("codex вернул некорректный JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("ошибка протокола codex: {0}")]
    Protocol(String),
}

pub fn run_worker(update_tx: Sender<WorkerUpdate>, command_rx: Receiver<WorkerCommand>) {
    loop {
        if update_tx.send(WorkerUpdate::Querying).is_err() {
            return;
        }

        match subscribe_to_usage(&update_tx, &command_rx) {
            Ok(()) => return,
            Err(error) => {
                if update_tx
                    .send(WorkerUpdate::Error {
                        message: error.to_string(),
                        at: Local::now().timestamp(),
                    })
                    .is_err()
                {
                    return;
                }
            }
        }

        match command_rx.recv_timeout(RECONNECT_DELAY) {
            Ok(WorkerCommand::Stop) | Err(RecvTimeoutError::Disconnected) => return,
            Err(RecvTimeoutError::Timeout) => {}
        }
    }
}

fn subscribe_to_usage(
    update_tx: &Sender<WorkerUpdate>,
    command_rx: &Receiver<WorkerCommand>,
) -> Result<(), CodexError> {
    let mut client = AppServerClient::start()?;

    client.send(&json!({
        "id": 1,
        "method": "initialize",
        "params": {
            "clientInfo": { "name": "codex-tray", "version": env!("CARGO_PKG_VERSION") },
            "capabilities": {}
        }
    }))?;
    client.read_response(1)?;
    client.send(&json!({ "method": "initialized" }))?;

    client.send(&json!({
        "id": 2,
        "method": "account/read",
        "params": { "refreshToken": false }
    }))?;
    let account = client.read_response(2)?;

    client.send(&json!({
        "id": 3,
        "method": "account/rateLimits/read"
    }))?;
    let mut limits = client.read_response(3)?;
    send_snapshot(update_tx, &account, &limits)?;

    loop {
        match command_rx.try_recv() {
            Ok(WorkerCommand::Stop) | Err(TryRecvError::Disconnected) => return Ok(()),
            Err(TryRecvError::Empty) => {}
        }

        match client.messages.recv_timeout(COMMAND_CHECK_INTERVAL) {
            Ok(Ok(message)) => {
                if message.get("method").and_then(Value::as_str)
                    == Some("account/rateLimits/updated")
                {
                    let params = message.get("params").ok_or_else(|| {
                        CodexError::Protocol("в уведомлении лимитов нет params".into())
                    })?;
                    merge_rate_limits_notification(&mut limits, params)?;
                    send_snapshot(update_tx, &account, &limits)?;
                }
            }
            Ok(Err(error)) => return Err(error),
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => {
                return Err(CodexError::Protocol(
                    "канал чтения app-server неожиданно закрылся".into(),
                ));
            }
        }
    }
}

fn send_snapshot(
    update_tx: &Sender<WorkerUpdate>,
    account: &Value,
    limits: &Value,
) -> Result<(), CodexError> {
    let snapshot = parse_snapshot(account, limits)?;
    update_tx
        .send(WorkerUpdate::Snapshot(snapshot))
        .map_err(|_| CodexError::Protocol("канал интерфейса закрыт".into()))
}

struct AppServerClient {
    child: Child,
    stdin: ChildStdin,
    messages: Receiver<Result<Value, CodexError>>,
}

impl AppServerClient {
    fn start() -> Result<Self, CodexError> {
        let mut command = Command::new("codex");
        command
            .args(["app-server", "--stdio"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(windows)]
        command.creation_flags(CREATE_NO_WINDOW);
        let mut child = command.spawn().map_err(CodexError::Spawn)?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| CodexError::Protocol("stdin app-server недоступен".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| CodexError::Protocol("stdout app-server недоступен".into()))?;
        let (message_tx, messages) = channel();
        thread::spawn(move || {
            let mut stdout = BufReader::new(stdout);
            let mut line = String::new();
            loop {
                line.clear();
                match stdout.read_line(&mut line) {
                    Ok(0) => {
                        let _ = message_tx.send(Err(CodexError::Protocol(
                            "app-server завершил поток событий".into(),
                        )));
                        return;
                    }
                    Ok(_) => match serde_json::from_str(&line) {
                        Ok(message) => {
                            if message_tx.send(Ok(message)).is_err() {
                                return;
                            }
                        }
                        Err(error) => {
                            let _ = message_tx.send(Err(CodexError::Json(error)));
                            return;
                        }
                    },
                    Err(error) => {
                        let _ = message_tx.send(Err(CodexError::Io(error)));
                        return;
                    }
                }
            }
        });

        Ok(Self {
            child,
            stdin,
            messages,
        })
    }

    fn send(&mut self, value: &Value) -> Result<(), CodexError> {
        serde_json::to_writer(&mut self.stdin, value)?;
        self.stdin.write_all(b"\n")?;
        self.stdin.flush()?;
        Ok(())
    }

    fn read_response(&self, expected_id: i64) -> Result<Value, CodexError> {
        loop {
            let value = self.messages.recv().map_err(channel_closed)??;
            if value.get("id").and_then(Value::as_i64) != Some(expected_id) {
                continue;
            }

            if let Some(error) = value.get("error") {
                return Err(CodexError::Protocol(error.to_string()));
            }
            return Ok(value);
        }
    }
}

fn channel_closed(_: RecvError) -> CodexError {
    CodexError::Protocol("канал чтения app-server неожиданно закрылся".into())
}

impl Drop for AppServerClient {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn merge_rate_limits_notification(
    limits_response: &mut Value,
    params: &Value,
) -> Result<(), CodexError> {
    let update = params
        .get("rateLimits")
        .and_then(Value::as_object)
        .ok_or_else(|| CodexError::Protocol("в уведомлении нет rateLimits".into()))?;
    let result = limits_response
        .get_mut("result")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| CodexError::Protocol("в снимке лимитов нет result".into()))?;

    merge_rate_limit_bucket(
        result.entry("rateLimits").or_insert_with(|| json!({})),
        update,
    )?;

    let limit_id = update
        .get("limitId")
        .and_then(Value::as_str)
        .unwrap_or("codex");
    if let Some(by_id) = result
        .get_mut("rateLimitsByLimitId")
        .and_then(Value::as_object_mut)
    {
        let bucket = by_id
            .entry(limit_id.to_owned())
            .or_insert_with(|| json!({}));
        merge_rate_limit_bucket(bucket, update)?;
    }

    Ok(())
}

fn merge_rate_limit_bucket(
    bucket: &mut Value,
    update: &serde_json::Map<String, Value>,
) -> Result<(), CodexError> {
    let bucket = bucket
        .as_object_mut()
        .ok_or_else(|| CodexError::Protocol("снимок лимита не является объектом".into()))?;

    for key in ["primary", "secondary", "rateLimitReachedType"] {
        if let Some(value) = update.get(key) {
            bucket.insert(key.into(), value.clone());
        }
    }
    for key in [
        "limitId",
        "limitName",
        "credits",
        "individualLimit",
        "spendControlReached",
        "planType",
    ] {
        if let Some(value) = update.get(key).filter(|value| !value.is_null()) {
            bucket.insert(key.into(), value.clone());
        }
    }
    Ok(())
}

fn parse_snapshot(account: &Value, limits: &Value) -> Result<UsageSnapshot, CodexError> {
    let result = limits
        .get("result")
        .ok_or_else(|| CodexError::Protocol("в ответе нет result".into()))?;

    let bucket = result
        .pointer("/rateLimitsByLimitId/codex")
        .or_else(|| result.get("rateLimits"))
        .ok_or_else(|| CodexError::Protocol("в ответе нет лимита codex".into()))?;

    let primary = bucket
        .get("primary")
        .filter(|value| !value.is_null())
        .ok_or_else(|| CodexError::Protocol("у лимита codex нет активного окна".into()))?;

    let used_percent = primary
        .get("usedPercent")
        .and_then(Value::as_u64)
        .ok_or_else(|| CodexError::Protocol("в окне нет usedPercent".into()))?
        .min(100) as u32;

    let plan_type = bucket
        .get("planType")
        .and_then(Value::as_str)
        .or_else(|| {
            account
                .pointer("/result/account/planType")
                .and_then(Value::as_str)
        })
        .map(str::to_owned);

    let credits = bucket.get("credits").filter(|value| !value.is_null());

    Ok(UsageSnapshot {
        used_percent,
        window_duration_mins: primary.get("windowDurationMins").and_then(Value::as_i64),
        resets_at: primary.get("resetsAt").and_then(Value::as_i64),
        plan_type,
        credit_balance: credits
            .and_then(|value| value.get("balance"))
            .and_then(Value::as_str)
            .map(str::to_owned),
        has_credits: credits
            .and_then(|value| value.get("hasCredits"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        unlimited_credits: credits
            .and_then(|value| value.get("unlimited"))
            .and_then(Value::as_bool)
            .unwrap_or(false),
        reset_credits: result
            .pointer("/rateLimitResetCredits/availableCount")
            .and_then(Value::as_i64)
            .unwrap_or(0),
        limit_reached_type: bucket
            .get("rateLimitReachedType")
            .and_then(Value::as_str)
            .map(str::to_owned),
        updated_at: Local::now().timestamp(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_current_codex_bucket() {
        let account = json!({
            "id": 2,
            "result": { "account": { "type": "chatgpt", "planType": "pro" } }
        });
        let limits = json!({
            "id": 3,
            "result": {
                "rateLimits": {},
                "rateLimitsByLimitId": {
                    "codex": {
                        "primary": {
                            "usedPercent": 17,
                            "windowDurationMins": 300,
                            "resetsAt": 1787903956
                        },
                        "credits": { "hasCredits": true, "unlimited": false, "balance": "12.5" },
                        "planType": "pro",
                        "rateLimitReachedType": null
                    }
                },
                "rateLimitResetCredits": { "availableCount": 2 }
            }
        });

        let snapshot = parse_snapshot(&account, &limits).expect("snapshot");
        assert_eq!(snapshot.used_percent, 17);
        assert_eq!(snapshot.window_duration_mins, Some(300));
        assert_eq!(snapshot.plan_type.as_deref(), Some("pro"));
        assert_eq!(snapshot.credit_balance.as_deref(), Some("12.5"));
        assert_eq!(snapshot.reset_credits, 2);
    }

    #[test]
    fn merges_sparse_rate_limit_notification_without_losing_account_metadata() {
        let account = json!({ "result": { "account": { "planType": "pro" } } });
        let mut limits = json!({
            "result": {
                "rateLimits": { "primary": { "usedPercent": 17 } },
                "rateLimitsByLimitId": {
                    "codex": {
                        "limitId": "codex",
                        "primary": { "usedPercent": 17, "windowDurationMins": 300 },
                        "credits": { "hasCredits": true, "unlimited": false, "balance": "12.5" },
                        "planType": "pro",
                        "rateLimitReachedType": "rate_limit_reached"
                    }
                }
            }
        });
        let notification = json!({
            "rateLimits": {
                "limitId": "codex",
                "primary": { "usedPercent": 31, "windowDurationMins": 300 },
                "credits": null,
                "planType": null,
                "rateLimitReachedType": null
            }
        });

        merge_rate_limits_notification(&mut limits, &notification).expect("merge");
        let snapshot = parse_snapshot(&account, &limits).expect("snapshot");
        assert_eq!(snapshot.used_percent, 31);
        assert_eq!(snapshot.plan_type.as_deref(), Some("pro"));
        assert_eq!(snapshot.credit_balance.as_deref(), Some("12.5"));
        assert_eq!(snapshot.limit_reached_type, None);
    }

    #[test]
    fn falls_back_to_legacy_bucket_and_account_plan() {
        let account = json!({ "result": { "account": { "planType": "plus" } } });
        let limits = json!({
            "result": {
                "rateLimits": {
                    "primary": { "usedPercent": 101, "windowDurationMins": 10080 },
                    "credits": null
                }
            }
        });

        let snapshot = parse_snapshot(&account, &limits).expect("snapshot");
        assert_eq!(snapshot.used_percent, 100);
        assert_eq!(snapshot.plan_type.as_deref(), Some("plus"));
        assert!(!snapshot.has_credits);
    }
}
