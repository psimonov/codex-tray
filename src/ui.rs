use std::{
    cell::RefCell,
    ffi::c_void,
    fs,
    mem::size_of,
    path::{Path, PathBuf},
    sync::mpsc::{Receiver, Sender, TryRecvError},
    time::{Duration, Instant},
};

use chrono::{DateTime, Local};
use serde_json::{Value, json};
use windows::{
    Win32::{
        Foundation::{
            COLORREF, ERROR_FILE_NOT_FOUND, HINSTANCE, HWND, LPARAM, LRESULT, POINT, RECT, SIZE,
            WPARAM,
        },
        Globalization::GetUserDefaultLocaleName,
        Graphics::{
            Dwm::{DWMWA_WINDOW_CORNER_PREFERENCE, DwmGetColorizationColor, DwmSetWindowAttribute},
            Gdi::{
                BeginPaint, CreateFontIndirectW, CreateSolidBrush, DeleteObject, EndPaint,
                FW_SEMIBOLD, FillRect, GetMonitorInfoW, GetStockObject, GetTextExtentPoint32W,
                HBRUSH, HFONT, HGDIOBJ, IntersectClipRect, InvalidateRect,
                MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow, PAINTSTRUCT, RestoreDC,
                SYSTEM_FONT, SaveDC, SelectObject, SetBkMode, SetPixelV, SetTextColor, TRANSPARENT,
                TextOutW,
            },
        },
        System::{
            Com::{
                COINIT_APARTMENTTHREADED, COINIT_DISABLE_OLE1DDE, CoInitializeEx, CoUninitialize,
            },
            LibraryLoader::GetModuleHandleW,
            Registry::{
                HKEY_CURRENT_USER, REG_SZ, RRF_RT_REG_DWORD, RRF_RT_REG_SZ, RegDeleteKeyValueW,
                RegGetValueW, RegSetKeyValueW,
            },
        },
        UI::{
            HiDpi::{
                DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, GetDpiForWindow,
                GetSystemMetricsForDpi, SetProcessDpiAwarenessContext, SystemParametersInfoForDpi,
            },
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW,
                NOTIFYICONIDENTIFIER, Shell_NotifyIconGetRect, Shell_NotifyIconW, ShellExecuteW,
            },
            WindowsAndMessaging::{
                AppendMenuW, CREATESTRUCTW, CW_USEDEFAULT, CreatePopupMenu, CreateWindowExW,
                DefWindowProcW, DestroyIcon, DestroyMenu, DestroyWindow, DispatchMessageW,
                GWLP_USERDATA, GetClientRect, GetCursorPos, GetMessageW, HICON, IDC_ARROW,
                IDI_APPLICATION, IMAGE_ICON, LR_DEFAULTCOLOR, LWA_ALPHA, LoadCursorW, LoadIconW,
                LoadImageW, MB_ICONERROR, MB_OK, MF_CHECKED, MF_POPUP, MF_SEPARATOR, MF_STRING,
                MF_UNCHECKED, MSG, MessageBoxW, NONCLIENTMETRICSW, PostQuitMessage,
                RegisterClassExW, SM_CXSMICON, SPI_GETNONCLIENTMETRICS, SW_HIDE, SW_SHOWNORMAL,
                SWP_NOACTIVATE, SWP_SHOWWINDOW, SetForegroundWindow, SetLayeredWindowAttributes,
                SetTimer, SetWindowLongPtrW, SetWindowPos, ShowWindow, TPM_BOTTOMALIGN,
                TPM_LEFTALIGN, TPM_RETURNCMD, TrackPopupMenu, WM_APP, WM_COMMAND, WM_CREATE,
                WM_DESTROY, WM_MOUSEMOVE, WM_NCCREATE, WM_PAINT, WM_RBUTTONUP, WM_TIMER,
                WNDCLASSEXW, WS_EX_LAYERED, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
                WS_POPUP,
            },
        },
    },
    core::{BOOL, PCWSTR, Result as WinResult, w},
};

use crate::codex::{UsageSnapshot, WorkerCommand, WorkerUpdate};

const WINDOW_CLASS: PCWSTR = w!("CodexTrayStatusWindow");
const WINDOW_WIDTH: i32 = 320;
const WINDOW_HEIGHT: i32 = 195;
const TRAY_ID: u32 = 1;
const TRAY_MESSAGE: u32 = WM_APP + 1;
const TIMER_ID: usize = 1;
const MENU_AUTOSTART: usize = 1001;
const MENU_EXIT: usize = 1002;
const MENU_REFRESH: usize = 1003;
const MENU_OPEN_FOLDER: usize = 1004;
const HOVER_HIDE_DELAY: Duration = Duration::from_millis(150);
const AUTOSTART_KEY: PCWSTR = w!("Software\\Microsoft\\Windows\\CurrentVersion\\Run");
const AUTOSTART_VALUE: PCWSTR = w!("Codex Tray");
const APP_ICON_RESOURCE: u16 = 1;
const STATUS_ICON_RESOURCES: &[(u32, u16)] = &[
    (0, 100),
    (5, 101),
    (25, 102),
    (50, 103),
    (75, 104),
    (95, 105),
    (100, 106),
];
const LOADING_ICON_RESOURCE: u16 = 107;
const ERROR_ICON_RESOURCE: u16 = 108;
const ACCOUNT_ICON_RESOURCE: u16 = 109;
const MISSING_ICON_RESOURCE: u16 = 110;
const MENU_LANGUAGE_BASE: usize = 1100;

const MENU_LANGUAGE: usize = 0;
const MENU_SYSTEM_LANGUAGE: usize = 1;
const MENU_REFRESH_NOW: usize = 2;
const MENU_OPEN_PROGRAM_FOLDER: usize = 3;
const MENU_START_WITH_WINDOWS: usize = 4;
const MENU_CLOSE: usize = 5;

const ROW_STATUS: usize = 0;
const ROW_REMAINING: usize = 1;
const ROW_USED: usize = 2;
const ROW_PLAN: usize = 3;
const ROW_WINDOW: usize = 4;
const ROW_RESET: usize = 5;
const ROW_CREDITS: usize = 6;
const ROW_UPDATED: usize = 7;

const STATUS_LOADING: usize = 0;
const STATUS_REFRESHING: usize = 1;
const STATUS_READY: usize = 2;
const STATUS_EXHAUSTED: usize = 3;
const STATUS_ACCOUNT_REQUIRED: usize = 4;
const STATUS_SUBSCRIPTION_REQUIRED: usize = 5;
const STATUS_CODEX_MISSING: usize = 6;
const STATUS_ERROR: usize = 7;

const UNIT_WEEK: usize = 0;
const UNIT_DAY: usize = 1;
const UNIT_HOUR: usize = 2;
const UNIT_MINUTE: usize = 3;

const MESSAGE_REFRESH_FAILED: usize = 0;
const MESSAGE_FOLDER_UNKNOWN: usize = 1;
const MESSAGE_EXECUTABLE_PATH_FAILED: usize = 2;
const MESSAGE_OPEN_FOLDER_FAILED: usize = 3;
const MESSAGE_AUTOSTART_FAILED: usize = 4;
const MESSAGE_CONFIG_READ_FAILED: usize = 5;
const MESSAGE_CONFIG_INVALID: usize = 6;
const MESSAGE_CONFIG_WRITE_FAILED: usize = 7;
const MESSAGE_START_FAILED: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Language {
    English,
    Spanish,
    French,
    Portuguese,
    German,
    Italian,
    Russian,
    SimplifiedChinese,
    Hindi,
    Arabic,
    Japanese,
    Korean,
}

const LANGUAGES: &[Language] = &[
    Language::English,
    Language::Spanish,
    Language::French,
    Language::Portuguese,
    Language::German,
    Language::Italian,
    Language::Russian,
    Language::SimplifiedChinese,
    Language::Hindi,
    Language::Arabic,
    Language::Japanese,
    Language::Korean,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum LanguagePreference {
    System,
    Selected(Language),
}

impl LanguagePreference {
    fn code(self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Selected(language) => language.code(),
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        if code.eq_ignore_ascii_case("system") {
            Some(Self::System)
        } else {
            Language::from_code(code).map(Self::Selected)
        }
    }

    fn effective(self) -> Language {
        match self {
            Self::System => system_language(),
            Self::Selected(language) => language,
        }
    }
}

impl Language {
    fn code(self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::Portuguese => "pt",
            Self::German => "de",
            Self::Italian => "it",
            Self::Russian => "ru",
            Self::SimplifiedChinese => "zh-CN",
            Self::Hindi => "hi",
            Self::Arabic => "ar",
            Self::Japanese => "ja",
            Self::Korean => "ko",
        }
    }

    fn from_code(code: &str) -> Option<Self> {
        LANGUAGES
            .iter()
            .copied()
            .find(|language| language.code().eq_ignore_ascii_case(code))
    }

    fn native_name(self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Spanish => "Español",
            Self::French => "Français",
            Self::Portuguese => "Português",
            Self::German => "Deutsch",
            Self::Italian => "Italiano",
            Self::Russian => "Русский",
            Self::SimplifiedChinese => "简体中文",
            Self::Hindi => "हिन्दी",
            Self::Arabic => "العربية",
            Self::Japanese => "日本語",
            Self::Korean => "한국어",
        }
    }

    fn texts(self) -> &'static Texts {
        match self {
            Self::English => &ENGLISH_TEXTS,
            Self::Spanish => &SPANISH_TEXTS,
            Self::French => &FRENCH_TEXTS,
            Self::Portuguese => &PORTUGUESE_TEXTS,
            Self::German => &GERMAN_TEXTS,
            Self::Italian => &ITALIAN_TEXTS,
            Self::Russian => &RUSSIAN_TEXTS,
            Self::SimplifiedChinese => &CHINESE_TEXTS,
            Self::Hindi => &HINDI_TEXTS,
            Self::Arabic => &ARABIC_TEXTS,
            Self::Japanese => &JAPANESE_TEXTS,
            Self::Korean => &KOREAN_TEXTS,
        }
    }
}

struct Texts {
    menu: [&'static str; 6],
    rows: [&'static str; 8],
    statuses: [&'static str; 8],
    units: [&'static str; 4],
    unknown: &'static str,
    unknown_window: &'static str,
    messages: [&'static str; 9],
}

const ENGLISH_TEXTS: Texts = Texts {
    menu: [
        "Language",
        "System language",
        "Refresh now",
        "Open application folder",
        "Start with Windows",
        "Close",
    ],
    rows: [
        "Status",
        "Remaining",
        "Used",
        "Plan",
        "Window",
        "Reset",
        "Credits",
        "Updated",
    ],
    statuses: [
        "Loading",
        "Refreshing",
        "Ready",
        "Limit exhausted",
        "Sign-in required",
        "No active access",
        "Codex not found",
        "Codex error",
    ],
    units: ["wk", "day", "hr", "min"],
    unknown: "unknown",
    unknown_window: "unknown window",
    messages: [
        "Failed to request a data refresh",
        "Could not determine the application folder",
        "Could not determine the application path",
        "Could not open the application folder",
        "Could not change Windows startup",
        "Could not read the configuration",
        "The configuration file is invalid",
        "Could not write the configuration",
        "Codex Tray could not start",
    ],
};

const SPANISH_TEXTS: Texts = Texts {
    menu: [
        "Idioma",
        "Idioma del sistema",
        "Actualizar ahora",
        "Abrir carpeta de la aplicación",
        "Iniciar con Windows",
        "Cerrar",
    ],
    rows: [
        "Estado",
        "Restante",
        "Usado",
        "Plan",
        "Ventana",
        "Restablecimiento",
        "Créditos",
        "Actualizado",
    ],
    statuses: [
        "Cargando",
        "Actualizando",
        "Listo",
        "Límite agotado",
        "Inicio de sesión requerido",
        "Sin acceso activo",
        "Codex no encontrado",
        "Error de Codex",
    ],
    units: ["sem.", "d", "h", "min"],
    unknown: "desconocido",
    unknown_window: "ventana desconocida",
    messages: [
        "No se pudo solicitar la actualización de datos",
        "No se pudo determinar la carpeta de la aplicación",
        "No se pudo determinar la ruta de la aplicación",
        "No se pudo abrir la carpeta de la aplicación",
        "No se pudo cambiar el inicio con Windows",
        "No se pudo leer la configuración",
        "El archivo de configuración no es válido",
        "No se pudo guardar la configuración",
        "Codex Tray no pudo iniciarse",
    ],
};

const FRENCH_TEXTS: Texts = Texts {
    menu: [
        "Langue",
        "Langue du système",
        "Actualiser maintenant",
        "Ouvrir le dossier de l’application",
        "Démarrer avec Windows",
        "Fermer",
    ],
    rows: [
        "État",
        "Restant",
        "Utilisé",
        "Forfait",
        "Fenêtre",
        "Réinitialisation",
        "Crédits",
        "Mis à jour",
    ],
    statuses: [
        "Chargement",
        "Actualisation",
        "Prêt",
        "Limite épuisée",
        "Connexion requise",
        "Aucun accès actif",
        "Codex introuvable",
        "Erreur Codex",
    ],
    units: ["sem.", "j", "h", "min"],
    unknown: "inconnu",
    unknown_window: "fenêtre inconnue",
    messages: [
        "Impossible de demander l’actualisation des données",
        "Impossible de déterminer le dossier de l’application",
        "Impossible de déterminer le chemin de l’application",
        "Impossible d’ouvrir le dossier de l’application",
        "Impossible de modifier le démarrage avec Windows",
        "Impossible de lire la configuration",
        "Le fichier de configuration n’est pas valide",
        "Impossible d’enregistrer la configuration",
        "Codex Tray n’a pas pu démarrer",
    ],
};

const PORTUGUESE_TEXTS: Texts = Texts {
    menu: [
        "Idioma",
        "Idioma do sistema",
        "Atualizar agora",
        "Abrir pasta do aplicativo",
        "Iniciar com o Windows",
        "Fechar",
    ],
    rows: [
        "Status",
        "Restante",
        "Usado",
        "Plano",
        "Janela",
        "Redefinição",
        "Créditos",
        "Atualizado",
    ],
    statuses: [
        "Carregando",
        "Atualizando",
        "Pronto",
        "Limite esgotado",
        "Login necessário",
        "Sem acesso ativo",
        "Codex não encontrado",
        "Erro do Codex",
    ],
    units: ["sem.", "d", "h", "min"],
    unknown: "desconhecido",
    unknown_window: "janela desconhecida",
    messages: [
        "Não foi possível solicitar a atualização dos dados",
        "Não foi possível determinar a pasta do aplicativo",
        "Não foi possível determinar o caminho do aplicativo",
        "Não foi possível abrir a pasta do aplicativo",
        "Não foi possível alterar a inicialização com o Windows",
        "Não foi possível ler a configuração",
        "O arquivo de configuração é inválido",
        "Não foi possível salvar a configuração",
        "O Codex Tray não pôde ser iniciado",
    ],
};

const GERMAN_TEXTS: Texts = Texts {
    menu: [
        "Sprache",
        "Systemsprache",
        "Jetzt aktualisieren",
        "Anwendungsordner öffnen",
        "Mit Windows starten",
        "Beenden",
    ],
    rows: [
        "Status",
        "Verbleibend",
        "Verwendet",
        "Tarif",
        "Zeitraum",
        "Zurücksetzung",
        "Guthaben",
        "Aktualisiert",
    ],
    statuses: [
        "Wird geladen",
        "Wird aktualisiert",
        "Bereit",
        "Limit ausgeschöpft",
        "Anmeldung erforderlich",
        "Kein aktiver Zugriff",
        "Codex nicht gefunden",
        "Codex-Fehler",
    ],
    units: ["Wo.", "Tg.", "Std.", "Min."],
    unknown: "unbekannt",
    unknown_window: "Zeitraum unbekannt",
    messages: [
        "Die Datenaktualisierung konnte nicht angefordert werden",
        "Der Anwendungsordner konnte nicht ermittelt werden",
        "Der Anwendungspfad konnte nicht ermittelt werden",
        "Der Anwendungsordner konnte nicht geöffnet werden",
        "Der Windows-Autostart konnte nicht geändert werden",
        "Die Konfiguration konnte nicht gelesen werden",
        "Die Konfigurationsdatei ist ungültig",
        "Die Konfiguration konnte nicht gespeichert werden",
        "Codex Tray konnte nicht gestartet werden",
    ],
};

const ITALIAN_TEXTS: Texts = Texts {
    menu: [
        "Lingua",
        "Lingua di sistema",
        "Aggiorna ora",
        "Apri cartella dell’applicazione",
        "Avvia con Windows",
        "Chiudi",
    ],
    rows: [
        "Stato",
        "Rimanente",
        "Usato",
        "Piano",
        "Finestra",
        "Ripristino",
        "Crediti",
        "Aggiornato",
    ],
    statuses: [
        "Caricamento",
        "Aggiornamento",
        "Pronto",
        "Limite esaurito",
        "Accesso richiesto",
        "Nessun accesso attivo",
        "Codex non trovato",
        "Errore Codex",
    ],
    units: ["sett.", "g", "ore", "min"],
    unknown: "sconosciuto",
    unknown_window: "finestra sconosciuta",
    messages: [
        "Impossibile richiedere l’aggiornamento dei dati",
        "Impossibile determinare la cartella dell’applicazione",
        "Impossibile determinare il percorso dell’applicazione",
        "Impossibile aprire la cartella dell’applicazione",
        "Impossibile modificare l’avvio con Windows",
        "Impossibile leggere la configurazione",
        "Il file di configurazione non è valido",
        "Impossibile salvare la configurazione",
        "Impossibile avviare Codex Tray",
    ],
};

const RUSSIAN_TEXTS: Texts = Texts {
    menu: [
        "Язык",
        "Системный язык",
        "Обновить сейчас",
        "Открыть папку с программой",
        "Запускать вместе с Windows",
        "Закрыть",
    ],
    rows: [
        "Статус",
        "Осталось",
        "Использовано",
        "Тариф",
        "Окно",
        "Сброс",
        "Кредиты",
        "Обновлено",
    ],
    statuses: [
        "Загрузка",
        "Обновление",
        "Готово",
        "Лимит исчерпан",
        "Требуется вход",
        "Нет активного доступа",
        "Codex не найден",
        "Ошибка Codex",
    ],
    units: ["нед.", "дн.", "ч", "мин"],
    unknown: "неизвестно",
    unknown_window: "окно неизвестно",
    messages: [
        "Не удалось запросить обновление данных",
        "Не удалось определить папку приложения",
        "Не удалось определить путь приложения",
        "Не удалось открыть папку приложения",
        "Не удалось изменить автозапуск Windows",
        "Не удалось прочитать конфигурацию",
        "Файл конфигурации некорректен",
        "Не удалось сохранить конфигурацию",
        "Codex Tray не удалось запустить",
    ],
};

const CHINESE_TEXTS: Texts = Texts {
    menu: [
        "语言",
        "系统语言",
        "立即刷新",
        "打开应用目录",
        "随 Windows 启动",
        "关闭",
    ],
    rows: [
        "状态",
        "剩余",
        "已用",
        "方案",
        "周期",
        "重置",
        "额度",
        "更新时间",
    ],
    statuses: [
        "正在加载",
        "正在刷新",
        "就绪",
        "额度已耗尽",
        "需要登录",
        "无有效访问权限",
        "未找到 Codex",
        "Codex 错误",
    ],
    units: ["周", "天", "小时", "分钟"],
    unknown: "未知",
    unknown_window: "周期未知",
    messages: [
        "无法请求刷新数据",
        "无法确定应用目录",
        "无法确定应用路径",
        "无法打开应用目录",
        "无法更改 Windows 自启动",
        "无法读取配置",
        "配置文件无效",
        "无法保存配置",
        "Codex Tray 无法启动",
    ],
};

const HINDI_TEXTS: Texts = Texts {
    menu: [
        "भाषा",
        "सिस्टम की भाषा",
        "अभी अपडेट करें",
        "ऐप का फ़ोल्डर खोलें",
        "Windows के साथ शुरू करें",
        "बंद करें",
    ],
    rows: [
        "स्थिति",
        "शेष",
        "उपयोग",
        "प्लान",
        "अवधि",
        "रीसेट",
        "क्रेडिट",
        "अपडेट",
    ],
    statuses: [
        "लोड हो रहा है",
        "अपडेट हो रहा है",
        "तैयार",
        "सीमा समाप्त",
        "लॉगिन आवश्यक",
        "सक्रिय पहुँच नहीं",
        "Codex नहीं मिला",
        "Codex त्रुटि",
    ],
    units: ["हफ़्ते", "दिन", "घं.", "मिनट"],
    unknown: "अज्ञात",
    unknown_window: "अवधि अज्ञात",
    messages: [
        "डेटा अपडेट का अनुरोध नहीं किया जा सका",
        "ऐप का फ़ोल्डर निर्धारित नहीं किया जा सका",
        "ऐप का पथ निर्धारित नहीं किया जा सका",
        "ऐप का फ़ोल्डर नहीं खोला जा सका",
        "Windows स्टार्टअप नहीं बदला जा सका",
        "कॉन्फ़िगरेशन पढ़ा नहीं जा सका",
        "कॉन्फ़िगरेशन फ़ाइल अमान्य है",
        "कॉन्फ़िगरेशन सहेजा नहीं जा सका",
        "Codex Tray शुरू नहीं हो सका",
    ],
};

const ARABIC_TEXTS: Texts = Texts {
    menu: [
        "اللغة",
        "لغة النظام",
        "تحديث الآن",
        "فتح مجلد التطبيق",
        "التشغيل مع Windows",
        "إغلاق",
    ],
    rows: [
        "الحالة",
        "المتبقي",
        "المستخدم",
        "الخطة",
        "النافذة",
        "إعادة الضبط",
        "الأرصدة",
        "آخر تحديث",
    ],
    statuses: [
        "جارٍ التحميل",
        "جارٍ التحديث",
        "جاهز",
        "نفدت الحصة",
        "تسجيل الدخول مطلوب",
        "لا يوجد وصول نشط",
        "لم يتم العثور على Codex",
        "خطأ في Codex",
    ],
    units: ["أسب.", "يوم", "س", "د"],
    unknown: "غير معروف",
    unknown_window: "نافذة غير معروفة",
    messages: [
        "تعذر طلب تحديث البيانات",
        "تعذر تحديد مجلد التطبيق",
        "تعذر تحديد مسار التطبيق",
        "تعذر فتح مجلد التطبيق",
        "تعذر تغيير التشغيل مع Windows",
        "تعذرت قراءة الإعدادات",
        "ملف الإعدادات غير صالح",
        "تعذر حفظ الإعدادات",
        "تعذر تشغيل Codex Tray",
    ],
};

const JAPANESE_TEXTS: Texts = Texts {
    menu: [
        "言語",
        "システム言語",
        "今すぐ更新",
        "アプリケーションフォルダーを開く",
        "Windows と同時に起動",
        "終了",
    ],
    rows: [
        "状態",
        "残り",
        "使用済み",
        "プラン",
        "期間",
        "リセット",
        "クレジット",
        "更新日時",
    ],
    statuses: [
        "読み込み中",
        "更新中",
        "準備完了",
        "上限に到達",
        "ログインが必要",
        "有効なアクセスなし",
        "Codex が見つかりません",
        "Codex エラー",
    ],
    units: ["週", "日", "時間", "分"],
    unknown: "不明",
    unknown_window: "期間不明",
    messages: [
        "データ更新を要求できませんでした",
        "アプリケーションフォルダーを特定できませんでした",
        "アプリケーションのパスを特定できませんでした",
        "アプリケーションフォルダーを開けませんでした",
        "Windows 自動起動を変更できませんでした",
        "設定を読み込めませんでした",
        "設定ファイルが無効です",
        "設定を保存できませんでした",
        "Codex Tray を起動できませんでした",
    ],
};

const KOREAN_TEXTS: Texts = Texts {
    menu: [
        "언어",
        "시스템 언어",
        "지금 새로 고침",
        "애플리케이션 폴더 열기",
        "Windows 시작 시 실행",
        "닫기",
    ],
    rows: [
        "상태",
        "남음",
        "사용됨",
        "요금제",
        "기간",
        "재설정",
        "크레딧",
        "업데이트",
    ],
    statuses: [
        "불러오는 중",
        "새로 고치는 중",
        "준비됨",
        "한도 소진",
        "로그인 필요",
        "활성 접근 권한 없음",
        "Codex를 찾을 수 없음",
        "Codex 오류",
    ],
    units: ["주", "일", "시간", "분"],
    unknown: "알 수 없음",
    unknown_window: "기간 알 수 없음",
    messages: [
        "데이터 새로 고침을 요청할 수 없습니다",
        "애플리케이션 폴더를 확인할 수 없습니다",
        "애플리케이션 경로를 확인할 수 없습니다",
        "애플리케이션 폴더를 열 수 없습니다",
        "Windows 시작 설정을 변경할 수 없습니다",
        "설정을 읽을 수 없습니다",
        "설정 파일이 올바르지 않습니다",
        "설정을 저장할 수 없습니다",
        "Codex Tray를 시작할 수 없습니다",
    ],
};

fn language_from_locale_name(locale: &str) -> Language {
    match locale
        .split(['-', '_'])
        .next()
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "es" => Language::Spanish,
        "fr" => Language::French,
        "pt" => Language::Portuguese,
        "de" => Language::German,
        "it" => Language::Italian,
        "ru" => Language::Russian,
        "zh" => Language::SimplifiedChinese,
        "hi" => Language::Hindi,
        "ar" => Language::Arabic,
        "ja" => Language::Japanese,
        "ko" => Language::Korean,
        _ => Language::English,
    }
}

fn system_language() -> Language {
    let mut locale = [0_u16; 85];
    let length = unsafe { GetUserDefaultLocaleName(&mut locale) };
    if length <= 1 {
        return Language::English;
    }
    language_from_locale_name(&String::from_utf16_lossy(&locale[..length as usize - 1]))
}

#[derive(Clone)]
struct Settings {
    path: PathBuf,
    language: LanguagePreference,
    start_with_windows: bool,
}

#[derive(Debug)]
enum SettingsError {
    ExecutablePath,
    Read,
    Invalid,
    Write,
    Autostart(u32),
}

impl SettingsError {
    fn localized(&self, texts: &Texts) -> String {
        match self {
            Self::ExecutablePath => texts.messages[MESSAGE_EXECUTABLE_PATH_FAILED].to_owned(),
            Self::Read => texts.messages[MESSAGE_CONFIG_READ_FAILED].to_owned(),
            Self::Invalid => texts.messages[MESSAGE_CONFIG_INVALID].to_owned(),
            Self::Write => texts.messages[MESSAGE_CONFIG_WRITE_FAILED].to_owned(),
            Self::Autostart(code) => {
                format!("{} ({code})", texts.messages[MESSAGE_AUTOSTART_FAILED])
            }
        }
    }
}

impl Settings {
    fn load_or_create() -> Result<Self, SettingsError> {
        let executable = std::env::current_exe().map_err(|_| SettingsError::ExecutablePath)?;
        let path = executable.with_file_name("codex-tray.json");
        let mut settings = if path.exists() {
            let content = fs::read_to_string(&path).map_err(|_| SettingsError::Read)?;
            Self::parse(path, &content)?
        } else {
            let settings = Self {
                path,
                language: LanguagePreference::System,
                start_with_windows: autostart_registry_present(),
            };
            settings.save()?;
            settings
        };

        set_autostart(settings.start_with_windows)?;
        settings.path = executable.with_file_name("codex-tray.json");
        Ok(settings)
    }

    fn parse(path: PathBuf, content: &str) -> Result<Self, SettingsError> {
        let value: Value = serde_json::from_str(content).map_err(|_| SettingsError::Invalid)?;
        let language_code = value
            .get("language")
            .and_then(Value::as_str)
            .ok_or(SettingsError::Invalid)?;
        let language =
            LanguagePreference::from_code(language_code).ok_or(SettingsError::Invalid)?;
        let start_with_windows = value
            .get("start_with_windows")
            .and_then(Value::as_bool)
            .ok_or(SettingsError::Invalid)?;
        Ok(Self {
            path,
            language,
            start_with_windows,
        })
    }

    fn save(&self) -> Result<(), SettingsError> {
        let content = self.serialize()?;
        fs::write(&self.path, content).map_err(|_| SettingsError::Write)
    }

    fn serialize(&self) -> Result<String, SettingsError> {
        let mut content = serde_json::to_string_pretty(&json!({
            "language": self.language.code(),
            "start_with_windows": self.start_with_windows,
        }))
        .map_err(|_| SettingsError::Invalid)?;
        content.push('\n');
        Ok(content)
    }

    fn texts(&self) -> &'static Texts {
        self.language.effective().texts()
    }
}

thread_local! {
    static APP: RefCell<Option<AppState>> = const { RefCell::new(None) };
}

struct AppState {
    updates: Receiver<WorkerUpdate>,
    commands: Sender<WorkerCommand>,
    snapshot: Option<UsageSnapshot>,
    last_error: Option<(String, i64)>,
    querying: bool,
    visible: bool,
    last_tray_hover: Option<Instant>,
    suppress_hover_until_leave: bool,
    tray_icon: Option<HICON>,
    tray_icon_resource: u16,
    settings: Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum UiStatus {
    Loading,
    Refreshing,
    Ready,
    Exhausted,
    AccountRequired,
    SubscriptionRequired,
    CodexMissing,
    Error,
}

struct ComApartment;

impl ComApartment {
    fn initialize() -> Option<Self> {
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED | COINIT_DISABLE_OLE1DDE)
                .is_ok()
                .then_some(Self)
        }
    }
}

impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize() };
    }
}

pub fn run(updates: Receiver<WorkerUpdate>, commands: Sender<WorkerCommand>) -> WinResult<()> {
    let settings = match Settings::load_or_create() {
        Ok(settings) => settings,
        Err(error) => {
            let texts = system_language().texts();
            let message = format!(
                "{}:\n\n{}",
                texts.messages[MESSAGE_START_FAILED],
                error.localized(texts)
            );
            unsafe { show_message_box(None, &message) };
            return Ok(());
        }
    };
    let _com = ComApartment::initialize();
    unsafe {
        let _ = SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
        let instance = GetModuleHandleW(None)?;
        let cursor = LoadCursorW(None, IDC_ARROW)?;
        let icon = LoadIconW(
            Some(HINSTANCE(instance.0)),
            resource_name(APP_ICON_RESOURCE),
        )
        .or_else(|_| LoadIconW(None, IDI_APPLICATION))?;

        let class = WNDCLASSEXW {
            cbSize: size_of::<WNDCLASSEXW>() as u32,
            hInstance: HINSTANCE(instance.0),
            lpszClassName: WINDOW_CLASS,
            lpfnWndProc: Some(window_proc),
            hCursor: cursor,
            hIcon: icon,
            hIconSm: icon,
            hbrBackground: HBRUSH::default(),
            ..Default::default()
        };

        if RegisterClassExW(&class) == 0 {
            return Err(windows::core::Error::from_thread());
        }

        let hwnd = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST | WS_EX_NOACTIVATE | WS_EX_LAYERED,
            WINDOW_CLASS,
            w!("Codex Tray"),
            WS_POPUP,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            WINDOW_WIDTH,
            WINDOW_HEIGHT,
            None,
            None,
            Some(instance.into()),
            None,
        )?;

        let tray_icon_resource = LOADING_ICON_RESOURCE;
        let tray_icon = load_status_icon(hwnd, tray_icon_resource)?;
        APP.with(|app| {
            *app.borrow_mut() = Some(AppState {
                updates,
                commands,
                snapshot: None,
                last_error: None,
                querying: true,
                visible: false,
                last_tray_hover: None,
                suppress_hover_until_leave: false,
                tray_icon: Some(tray_icon),
                tray_icon_resource,
                settings,
            });
        });

        add_tray_icon(hwnd, tray_icon)?;
        apply_rounded_corners(hwnd);
        apply_system_transparency(hwnd);
        position_near_tray(hwnd, false);
        SetTimer(Some(hwnd), TIMER_ID, 100, None);

        let mut message = MSG::default();
        while GetMessageW(&mut message, None, 0, 0).as_bool() {
            let _ = windows::Win32::UI::WindowsAndMessaging::TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }

    Ok(())
}

pub fn startup_error_message(error: &windows::core::Error) -> String {
    let texts = system_language().texts();
    format!("{}:\n\n{error}", texts.messages[MESSAGE_START_FAILED])
}

unsafe extern "system" fn window_proc(
    hwnd: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match message {
            WM_NCCREATE => {
                let create = lparam.0 as *const CREATESTRUCTW;
                if !create.is_null() {
                    SetWindowLongPtrW(hwnd, GWLP_USERDATA, (*create).lpCreateParams as isize);
                }
                LRESULT(1)
            }
            WM_CREATE => LRESULT(0),
            WM_TIMER if wparam.0 == TIMER_ID => {
                drain_updates(hwnd);
                update_hover_visibility(hwnd);
                LRESULT(0)
            }
            TRAY_MESSAGE => {
                let event = (lparam.0 as u32) & 0xffff;
                if event == WM_MOUSEMOVE {
                    show_hover_window(hwnd);
                } else if event == WM_RBUTTONUP {
                    hide_hover_window(hwnd);
                    show_context_menu(hwnd);
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                handle_menu_command(hwnd, wparam.0 & 0xffff);
                LRESULT(0)
            }
            WM_PAINT => {
                paint(hwnd);
                LRESULT(0)
            }
            WM_DESTROY => {
                let tray_icon = APP.with(|app| {
                    let mut app = app.borrow_mut();
                    if let Some(state) = app.as_mut() {
                        let _ = state.commands.send(WorkerCommand::Stop);
                        state.tray_icon.take()
                    } else {
                        None
                    }
                });
                remove_tray_icon(hwnd);
                if let Some(icon) = tray_icon {
                    let _ = DestroyIcon(icon);
                }
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, message, wparam, lparam),
        }
    }
}

unsafe fn drain_updates(hwnd: HWND) {
    unsafe {
        let mut changed = false;
        APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else { return };
            loop {
                match state.updates.try_recv() {
                    Ok(WorkerUpdate::Querying) => {
                        state.querying = true;
                        changed = true;
                    }
                    Ok(WorkerUpdate::Snapshot(snapshot)) => {
                        state.snapshot = Some(snapshot);
                        state.last_error = None;
                        state.querying = false;
                        changed = true;
                    }
                    Ok(WorkerUpdate::Error { message, at }) => {
                        state.last_error = Some((message, at));
                        state.querying = false;
                        changed = true;
                    }
                    Err(TryRecvError::Empty | TryRecvError::Disconnected) => break,
                }
            }
        });

        if changed {
            update_tray_visual(hwnd);
            let _ = InvalidateRect(Some(hwnd), None, false);
        }
    }
}

unsafe fn paint(hwnd: HWND) {
    unsafe {
        let mut paint = PAINTSTRUCT::default();
        let hdc = BeginPaint(hwnd, &mut paint);
        let mut client = RECT::default();
        let _ = GetClientRect(hwnd, &mut client);

        let palette = system_palette();
        fill_rect(hdc, &client, palette.background);

        let (snapshot, error, querying, texts) = APP.with(|app| {
            let app = app.borrow();
            let state = app.as_ref();
            (
                state.and_then(|value| value.snapshot.clone()),
                state.and_then(|value| value.last_error.clone()),
                state.is_some_and(|value| value.querying),
                state
                    .map(|value| value.settings.texts())
                    .unwrap_or(&ENGLISH_TEXTS),
            )
        });
        let status = ui_status(snapshot.as_ref(), error.as_ref(), querying);

        if let Some(snapshot) = snapshot {
            draw_snapshot(hwnd, hdc, &snapshot, status, error.as_ref(), texts);
        } else {
            draw_loading(hwnd, hdc, status, error.as_ref(), texts);
        }

        let _ = EndPaint(hwnd, &paint);
    }
}

unsafe fn draw_snapshot(
    hwnd: HWND,
    hdc: windows::Win32::Graphics::Gdi::HDC,
    snapshot: &UsageSnapshot,
    status: UiStatus,
    _error: Option<&(String, i64)>,
    texts: &'static Texts,
) {
    unsafe {
        let percent_font = create_system_font(hwnd, true);
        let body_font = create_system_font(hwnd, false);
        SetBkMode(hdc, TRANSPARENT);

        let track = RECT {
            left: scale(hwnd, 10),
            top: scale(hwnd, 9),
            right: client_width(hwnd) - scale(hwnd, 10),
            bottom: scale(hwnd, 29),
        };
        let corner_radius = scale(hwnd, 3);
        let palette = system_palette();
        let panel_color = palette.background;
        let track_color = palette.track;
        fill_rounded_rect(hdc, &track, track_color, panel_color, corner_radius);

        let remaining_percent = 100_u32.saturating_sub(snapshot.used_percent);
        let fill_width = ((track.right - track.left) * remaining_percent as i32) / 100;
        let fill_right = track.left + fill_width;
        if fill_width > 0 {
            let fill = RECT {
                right: fill_right,
                ..track
            };
            let fill_radius = corner_radius.min(fill_width / 2);
            fill_rounded_rect(
                hdc,
                &fill,
                remaining_color(remaining_percent),
                track_color,
                fill_radius,
            );
        }

        select_font(hdc, percent_font);
        draw_contrast_percent(
            hdc,
            &track,
            fill_right,
            remaining_percent,
            remaining_color(remaining_percent),
            palette,
        );

        let rows = snapshot_rows(snapshot, status, texts);
        draw_rows(hwnd, hdc, percent_font, body_font, &rows, status);

        let _ = SelectObject(hdc, GetStockObject(SYSTEM_FONT));
        let _ = DeleteObject(HGDIOBJ(percent_font.0));
        let _ = DeleteObject(HGDIOBJ(body_font.0));
    }
}

unsafe fn draw_loading(
    hwnd: HWND,
    hdc: windows::Win32::Graphics::Gdi::HDC,
    status: UiStatus,
    error: Option<&(String, i64)>,
    texts: &'static Texts,
) {
    unsafe {
        let title_font = create_system_font(hwnd, true);
        let body_font = create_system_font(hwnd, false);
        SetBkMode(hdc, TRANSPARENT);
        let track = RECT {
            left: scale(hwnd, 10),
            top: scale(hwnd, 9),
            right: client_width(hwnd) - scale(hwnd, 10),
            bottom: scale(hwnd, 29),
        };
        fill_rounded_rect(
            hdc,
            &track,
            system_palette().track,
            system_palette().background,
            scale(hwnd, 3),
        );
        select_font(hdc, title_font);
        draw_centered_text(
            hdc,
            &track,
            status_title(status, texts),
            status_color(status),
        );

        let updated = error
            .map(|(_, at)| format_datetime(*at, texts))
            .unwrap_or_else(|| "—".into());
        let rows = vec![
            (
                texts.rows[ROW_STATUS],
                status_title(status, texts).to_owned(),
            ),
            (texts.rows[ROW_REMAINING], "—".into()),
            (texts.rows[ROW_USED], "—".into()),
            (texts.rows[ROW_PLAN], "—".into()),
            (texts.rows[ROW_WINDOW], "—".into()),
            (texts.rows[ROW_RESET], "—".into()),
            (texts.rows[ROW_CREDITS], "—".into()),
            (texts.rows[ROW_UPDATED], updated),
        ];
        draw_rows(hwnd, hdc, title_font, body_font, &rows, status);
        let _ = SelectObject(hdc, GetStockObject(SYSTEM_FONT));
        let _ = DeleteObject(HGDIOBJ(title_font.0));
        let _ = DeleteObject(HGDIOBJ(body_font.0));
    }
}

fn snapshot_rows(
    snapshot: &UsageSnapshot,
    status: UiStatus,
    texts: &'static Texts,
) -> Vec<(&'static str, String)> {
    let remaining = 100_u32.saturating_sub(snapshot.used_percent);
    let reset = snapshot
        .resets_at
        .map(|timestamp| format_datetime(timestamp, texts))
        .unwrap_or_else(|| texts.unknown.into());
    vec![
        (
            texts.rows[ROW_STATUS],
            status_title(status, texts).to_owned(),
        ),
        (texts.rows[ROW_REMAINING], format!("{remaining}%")),
        (texts.rows[ROW_USED], format!("{}%", snapshot.used_percent)),
        (
            texts.rows[ROW_PLAN],
            snapshot
                .plan_type
                .clone()
                .unwrap_or_else(|| texts.unknown.into()),
        ),
        (
            texts.rows[ROW_WINDOW],
            format_window(snapshot.window_duration_mins, texts),
        ),
        (texts.rows[ROW_RESET], reset),
        (texts.rows[ROW_CREDITS], credits_text(snapshot)),
        (
            texts.rows[ROW_UPDATED],
            format_datetime(snapshot.updated_at, texts),
        ),
    ]
}

unsafe fn draw_rows(
    hwnd: HWND,
    hdc: windows::Win32::Graphics::Gdi::HDC,
    key_font: HFONT,
    value_font: HFONT,
    rows: &[(&str, String)],
    status: UiStatus,
) {
    unsafe {
        let palette = system_palette();
        let key_x = scale(hwnd, 10);
        select_font(hdc, key_font);
        let key_width = rows
            .iter()
            .map(|(key, _)| {
                let key = wide(&format!("{key}:"));
                let mut size = SIZE::default();
                let _ = GetTextExtentPoint32W(hdc, &key[..key.len() - 1], &mut size);
                size.cx
            })
            .max()
            .unwrap_or_default();
        let value_x =
            (key_x + key_width + scale(hwnd, 10)).min(client_width(hwnd) - scale(hwnd, 118));
        let first_y = scale(hwnd, 40);
        let row_height = scale(hwnd, 18);
        for (index, (key, value)) in rows.iter().enumerate() {
            let y = first_y + index as i32 * row_height;
            select_font(hdc, key_font);
            SetTextColor(hdc, palette.key);
            text_out(hdc, key_x, y, &format!("{key}:"));
            select_font(hdc, value_font);
            SetTextColor(
                hdc,
                if index == 0 {
                    status_color(status)
                } else {
                    palette.value
                },
            );
            text_out(hdc, value_x, y, value);
        }
    }
}

unsafe fn draw_centered_text(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: &RECT,
    value: &str,
    color: COLORREF,
) {
    unsafe {
        let text = wide(value);
        let visible = &text[..text.len() - 1];
        let mut size = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, visible, &mut size);
        SetTextColor(hdc, color);
        let _ = TextOutW(
            hdc,
            rect.left + (rect.right - rect.left - size.cx) / 2,
            rect.top + (rect.bottom - rect.top - size.cy) / 2,
            visible,
        );
    }
}

unsafe fn draw_contrast_percent(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    track: &RECT,
    fill_right: i32,
    percent: u32,
    fill_color: COLORREF,
    palette: SystemPalette,
) {
    unsafe {
        let text = wide(&format!("{percent}%"));
        let visible_text = &text[..text.len() - 1];
        let mut size = SIZE::default();
        let _ = GetTextExtentPoint32W(hdc, visible_text, &mut size);
        let x = track.left + (track.right - track.left - size.cx) / 2;
        let y = track.top + (track.bottom - track.top - size.cy) / 2;

        if fill_right < track.right {
            let saved = SaveDC(hdc);
            IntersectClipRect(hdc, fill_right, track.top, track.right, track.bottom);
            SetTextColor(hdc, palette.value);
            let _ = TextOutW(hdc, x, y, visible_text);
            let _ = RestoreDC(hdc, saved);
        }

        if fill_right > track.left {
            let saved = SaveDC(hdc);
            IntersectClipRect(hdc, track.left, track.top, fill_right, track.bottom);
            SetTextColor(hdc, contrast_color(fill_color));
            let _ = TextOutW(hdc, x, y, visible_text);
            let _ = RestoreDC(hdc, saved);
        }
    }
}

unsafe fn add_tray_icon(hwnd: HWND, icon: HICON) -> WinResult<()> {
    unsafe {
        let mut data = tray_data(hwnd);
        data.uFlags = NIF_MESSAGE | NIF_ICON;
        data.uCallbackMessage = TRAY_MESSAGE;
        data.hIcon = icon;
        if !Shell_NotifyIconW(NIM_ADD, &data).as_bool() {
            return Err(windows::core::Error::from_thread());
        }
        Ok(())
    }
}

unsafe fn update_tray_visual(hwnd: HWND) {
    unsafe {
        let desired_resource = APP.with(|app| {
            app.borrow()
                .as_ref()
                .map(tray_resource_for_state)
                .unwrap_or(LOADING_ICON_RESOURCE)
        });

        let current_resource = APP.with(|app| {
            app.borrow()
                .as_ref()
                .map(|state| state.tray_icon_resource)
                .unwrap_or(desired_resource)
        });
        let replacement = if desired_resource != current_resource {
            load_status_icon(hwnd, desired_resource).ok()
        } else {
            None
        };

        let mut icon_for_update = None;
        let mut old_icon = None;
        APP.with(|app| {
            let mut app = app.borrow_mut();
            if let Some(state) = app.as_mut() {
                if let Some(icon) = replacement {
                    old_icon = state.tray_icon.replace(icon);
                    state.tray_icon_resource = desired_resource;
                }
                icon_for_update = state.tray_icon;
            }
        });

        let mut data = tray_data(hwnd);
        if let Some(icon) = icon_for_update {
            data.uFlags = NIF_ICON;
            data.hIcon = icon;
            let _ = Shell_NotifyIconW(NIM_MODIFY, &data);
        }
        if let Some(icon) = old_icon {
            let _ = DestroyIcon(icon);
        }
    }
}

unsafe fn remove_tray_icon(hwnd: HWND) {
    unsafe {
        let data = tray_data(hwnd);
        let _ = Shell_NotifyIconW(NIM_DELETE, &data);
    }
}

fn tray_data(hwnd: HWND) -> NOTIFYICONDATAW {
    NOTIFYICONDATAW {
        cbSize: size_of::<NOTIFYICONDATAW>() as u32,
        hWnd: hwnd,
        uID: TRAY_ID,
        ..Default::default()
    }
}

unsafe fn show_hover_window(hwnd: HWND) {
    unsafe {
        let mut cursor = POINT::default();
        if GetCursorPos(&mut cursor).is_err()
            || !tray_icon_rect(hwnd).is_some_and(|rect| point_in_rect(cursor, rect))
        {
            return;
        }
        APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else { return };
            if state.suppress_hover_until_leave {
                return;
            }
            state.last_tray_hover = Some(Instant::now());
            if !state.visible {
                state.visible = true;
                position_near_tray(hwnd, true);
            }
        });
    }
}

unsafe fn update_hover_visibility(hwnd: HWND) {
    unsafe {
        let mut cursor = POINT::default();
        let cursor_over_icon = GetCursorPos(&mut cursor).is_ok()
            && tray_icon_rect(hwnd).is_some_and(|rect| point_in_rect(cursor, rect));

        APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else { return };
            if cursor_over_icon {
                if !state.suppress_hover_until_leave {
                    state.last_tray_hover = Some(Instant::now());
                }
                return;
            }
            state.suppress_hover_until_leave = false;
            if state.visible
                && state
                    .last_tray_hover
                    .is_some_and(|hover| hover.elapsed() >= HOVER_HIDE_DELAY)
            {
                state.visible = false;
                let _ = ShowWindow(hwnd, SW_HIDE);
            }
        });
    }
}

unsafe fn hide_hover_window(hwnd: HWND) {
    unsafe {
        APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else { return };
            state.visible = false;
            state.last_tray_hover = None;
            state.suppress_hover_until_leave = true;
            let _ = ShowWindow(hwnd, SW_HIDE);
        });
    }
}

unsafe fn tray_icon_rect(hwnd: HWND) -> Option<RECT> {
    unsafe {
        let identifier = NOTIFYICONIDENTIFIER {
            cbSize: size_of::<NOTIFYICONIDENTIFIER>() as u32,
            hWnd: hwnd,
            uID: TRAY_ID,
            ..Default::default()
        };
        Shell_NotifyIconGetRect(&identifier).ok()
    }
}

fn point_in_rect(point: POINT, rect: RECT) -> bool {
    point.x >= rect.left && point.x < rect.right && point.y >= rect.top && point.y < rect.bottom
}

unsafe fn show_context_menu(hwnd: HWND) {
    unsafe {
        let menu = match CreatePopupMenu() {
            Ok(menu) => menu,
            Err(_) => return,
        };
        let language_menu = match CreatePopupMenu() {
            Ok(menu) => menu,
            Err(_) => {
                let _ = DestroyMenu(menu);
                return;
            }
        };
        let (language_preference, start_with_windows, texts) = APP.with(|app| {
            let app = app.borrow();
            app.as_ref()
                .map(|state| {
                    (
                        state.settings.language,
                        state.settings.start_with_windows,
                        state.settings.texts(),
                    )
                })
                .unwrap_or((LanguagePreference::System, false, &ENGLISH_TEXTS))
        });

        let system_language = wide(texts.menu[MENU_SYSTEM_LANGUAGE]);
        let system_state = if language_preference == LanguagePreference::System {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        let _ = AppendMenuW(
            language_menu,
            MF_STRING | system_state,
            MENU_LANGUAGE_BASE,
            PCWSTR(system_language.as_ptr()),
        );
        let _ = AppendMenuW(language_menu, MF_SEPARATOR, 0, None);
        let mut language_names = Vec::with_capacity(LANGUAGES.len());
        for (index, language) in LANGUAGES.iter().copied().enumerate() {
            let name = wide(language.native_name());
            let state = if language_preference == LanguagePreference::Selected(language) {
                MF_CHECKED
            } else {
                MF_UNCHECKED
            };
            let _ = AppendMenuW(
                language_menu,
                MF_STRING | state,
                MENU_LANGUAGE_BASE + index + 1,
                PCWSTR(name.as_ptr()),
            );
            language_names.push(name);
        }

        let language = wide(texts.menu[MENU_LANGUAGE]);
        let refresh = wide(texts.menu[MENU_REFRESH_NOW]);
        let open_folder = wide(texts.menu[MENU_OPEN_PROGRAM_FOLDER]);
        let autostart = wide(texts.menu[MENU_START_WITH_WINDOWS]);
        let close = wide(texts.menu[MENU_CLOSE]);
        let autostart_state = if start_with_windows {
            MF_CHECKED
        } else {
            MF_UNCHECKED
        };
        let _ = AppendMenuW(
            menu,
            MF_POPUP,
            language_menu.0 as usize,
            PCWSTR(language.as_ptr()),
        );
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(menu, MF_STRING, MENU_REFRESH, PCWSTR(refresh.as_ptr()));
        let _ = AppendMenuW(
            menu,
            MF_STRING,
            MENU_OPEN_FOLDER,
            PCWSTR(open_folder.as_ptr()),
        );
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(
            menu,
            MF_STRING | autostart_state,
            MENU_AUTOSTART,
            PCWSTR(autostart.as_ptr()),
        );
        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
        let _ = AppendMenuW(menu, MF_STRING, MENU_EXIT, PCWSTR(close.as_ptr()));

        let mut cursor = POINT::default();
        let _ = GetCursorPos(&mut cursor);
        let _ = SetForegroundWindow(hwnd);
        let command = TrackPopupMenu(
            menu,
            TPM_LEFTALIGN | TPM_BOTTOMALIGN | TPM_RETURNCMD,
            cursor.x,
            cursor.y,
            None,
            hwnd,
            None,
        );
        let _ = DestroyMenu(menu);
        if command.0 != 0 {
            handle_menu_command(hwnd, command.0 as usize);
        }
    }
}

unsafe fn handle_menu_command(hwnd: HWND, command: usize) {
    unsafe {
        if (MENU_LANGUAGE_BASE..=MENU_LANGUAGE_BASE + LANGUAGES.len()).contains(&command) {
            let preference = if command == MENU_LANGUAGE_BASE {
                LanguagePreference::System
            } else {
                LanguagePreference::Selected(LANGUAGES[command - MENU_LANGUAGE_BASE - 1])
            };
            change_language(hwnd, preference);
            return;
        }

        match command {
            MENU_REFRESH => request_refresh(hwnd),
            MENU_OPEN_FOLDER => {
                if let Err(error) = open_program_folder(hwnd, current_texts()) {
                    show_ui_error(hwnd, &error);
                }
            }
            MENU_AUTOSTART => toggle_autostart(hwnd),
            MENU_EXIT => {
                let _ = DestroyWindow(hwnd);
            }
            _ => {}
        }
    }
}

unsafe fn request_refresh(hwnd: HWND) {
    unsafe {
        let sent = APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else {
                return false;
            };
            if state.commands.send(WorkerCommand::Refresh).is_err() {
                return false;
            }
            state.querying = true;
            state.last_error = None;
            true
        });

        if sent {
            update_tray_visual(hwnd);
            let _ = InvalidateRect(Some(hwnd), None, false);
        } else {
            show_ui_error(hwnd, current_texts().messages[MESSAGE_REFRESH_FAILED]);
        }
    }
}

unsafe fn change_language(hwnd: HWND, preference: LanguagePreference) {
    unsafe {
        let result = APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else {
                return Ok(());
            };
            if state.settings.language == preference {
                return Ok(());
            }
            let previous = state.settings.language;
            let previous_texts = state.settings.texts();
            state.settings.language = preference;
            if let Err(error) = state.settings.save() {
                state.settings.language = previous;
                return Err(error.localized(previous_texts));
            }
            Ok(())
        });

        match result {
            Ok(()) => {
                let _ = InvalidateRect(Some(hwnd), None, false);
            }
            Err(error) => show_ui_error(hwnd, &error),
        }
    }
}

unsafe fn toggle_autostart(hwnd: HWND) {
    unsafe {
        let result = APP.with(|app| {
            let mut app = app.borrow_mut();
            let Some(state) = app.as_mut() else {
                return Ok(());
            };
            let previous = state.settings.start_with_windows;
            let desired = !previous;
            let texts = state.settings.texts();
            set_autostart(desired).map_err(|error| error.localized(texts))?;
            state.settings.start_with_windows = desired;
            if let Err(error) = state.settings.save() {
                state.settings.start_with_windows = previous;
                let _ = set_autostart(previous);
                return Err(error.localized(texts));
            }
            Ok(())
        });
        if let Err(error) = result {
            show_ui_error(hwnd, &error);
        }
    }
}

fn executable_directory(executable: &Path) -> Option<&Path> {
    executable
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
}

unsafe fn open_program_folder(hwnd: HWND, texts: &Texts) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| {
        format!(
            "{}: {error}",
            texts.messages[MESSAGE_EXECUTABLE_PATH_FAILED]
        )
    })?;
    let directory = executable_directory(&executable)
        .ok_or_else(|| texts.messages[MESSAGE_FOLDER_UNKNOWN].to_owned())?;
    let directory = wide(&directory.to_string_lossy());
    let result = unsafe {
        ShellExecuteW(
            Some(hwnd),
            w!("open"),
            PCWSTR(directory.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };

    if result.0 as isize > 32 {
        Ok(())
    } else {
        Err(format!(
            "{} ({}).",
            texts.messages[MESSAGE_OPEN_FOLDER_FAILED], result.0 as isize
        ))
    }
}

fn autostart_registry_present() -> bool {
    let mut size = 0_u32;
    unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            AUTOSTART_KEY,
            AUTOSTART_VALUE,
            RRF_RT_REG_SZ,
            None,
            None,
            Some(&mut size),
        )
        .is_ok()
    }
}

fn set_autostart(enabled: bool) -> Result<(), SettingsError> {
    let result = unsafe {
        if enabled {
            let executable = std::env::current_exe().map_err(|_| SettingsError::ExecutablePath)?;
            let command = wide(&quote_executable(&executable));
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                AUTOSTART_KEY,
                AUTOSTART_VALUE,
                REG_SZ.0,
                Some(command.as_ptr().cast()),
                (command.len() * size_of::<u16>()) as u32,
            )
        } else {
            RegDeleteKeyValueW(HKEY_CURRENT_USER, AUTOSTART_KEY, AUTOSTART_VALUE)
        }
    };

    if result.is_ok() || (!enabled && result.0 as u32 == ERROR_FILE_NOT_FOUND.0) {
        Ok(())
    } else {
        Err(SettingsError::Autostart(result.0))
    }
}

fn quote_executable(path: &Path) -> String {
    format!("\"{}\"", path.to_string_lossy())
}

fn current_texts() -> &'static Texts {
    APP.with(|app| {
        app.borrow()
            .as_ref()
            .map(|state| state.settings.texts())
            .unwrap_or(&ENGLISH_TEXTS)
    })
}

unsafe fn show_ui_error(hwnd: HWND, message: &str) {
    unsafe { show_message_box(Some(hwnd), message) }
}

unsafe fn show_message_box(hwnd: Option<HWND>, message: &str) {
    unsafe {
        let message = wide(message);
        let _ = MessageBoxW(
            hwnd,
            PCWSTR(message.as_ptr()),
            w!("Codex Tray"),
            MB_OK | MB_ICONERROR,
        );
    }
}

unsafe fn position_near_tray(hwnd: HWND, show: bool) {
    unsafe {
        let monitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
        let mut info = MONITORINFO {
            cbSize: size_of::<MONITORINFO>() as u32,
            ..Default::default()
        };
        let work = if GetMonitorInfoW(monitor, &mut info).as_bool() {
            info.rcWork
        } else {
            RECT {
                left: 0,
                top: 0,
                right: 1920,
                bottom: 1080,
            }
        };

        let flags = if show {
            SWP_NOACTIVATE | SWP_SHOWWINDOW
        } else {
            SWP_NOACTIVATE
        };
        let height = scale(hwnd, WINDOW_HEIGHT);
        let margin = scale(hwnd, 16);
        let width = scale(hwnd, WINDOW_WIDTH);
        let _ = SetWindowPos(
            hwnd,
            Some(windows::Win32::UI::WindowsAndMessaging::HWND_TOPMOST),
            work.right - width - margin,
            work.bottom - height - margin,
            width,
            height,
            flags,
        );
    }
}

unsafe fn apply_rounded_corners(hwnd: HWND) {
    unsafe {
        let preference: u32 = 2;
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const u32 as *const c_void,
            size_of::<u32>() as u32,
        );
    }
}

unsafe fn apply_system_transparency(hwnd: HWND) {
    unsafe {
        let transparency_enabled = read_personalization_dword("EnableTransparency") != Some(0);
        let alpha = if transparency_enabled { 238 } else { 255 };
        let _ = SetLayeredWindowAttributes(hwnd, COLORREF(0), alpha, LWA_ALPHA);
    }
}

#[derive(Clone, Copy)]
struct SystemPalette {
    background: COLORREF,
    track: COLORREF,
    key: COLORREF,
    value: COLORREF,
}

fn system_palette() -> SystemPalette {
    let light = read_personalization_dword("AppsUseLightTheme").is_some_and(|value| value != 0);
    if light {
        SystemPalette {
            background: rgb(243, 243, 243),
            track: rgb(215, 215, 215),
            key: rgb(90, 90, 90),
            value: rgb(24, 24, 24),
        }
    } else {
        SystemPalette {
            background: rgb(32, 32, 32),
            track: rgb(58, 58, 58),
            key: rgb(175, 175, 175),
            value: rgb(245, 245, 245),
        }
    }
}

fn read_personalization_dword(value_name: &str) -> Option<u32> {
    let value_name = wide(value_name);
    let mut value = 0_u32;
    let mut size = size_of::<u32>() as u32;
    let result = unsafe {
        RegGetValueW(
            HKEY_CURRENT_USER,
            w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
            PCWSTR(value_name.as_ptr()),
            RRF_RT_REG_DWORD,
            None,
            Some(&mut value as *mut u32 as *mut c_void),
            Some(&mut size),
        )
    };
    result.is_ok().then_some(value)
}

fn contrast_color(background: COLORREF) -> COLORREF {
    let red = background.0 & 0xff;
    let green = (background.0 >> 8) & 0xff;
    let blue = (background.0 >> 16) & 0xff;
    if red * 299 + green * 587 + blue * 114 >= 150_000 {
        rgb(18, 18, 18)
    } else {
        rgb(250, 250, 250)
    }
}

unsafe fn client_width(hwnd: HWND) -> i32 {
    unsafe {
        let mut client = RECT::default();
        if GetClientRect(hwnd, &mut client).is_ok() {
            client.right - client.left
        } else {
            scale(hwnd, WINDOW_WIDTH)
        }
    }
}

fn status_title(status: UiStatus, texts: &'static Texts) -> &'static str {
    match status {
        UiStatus::Loading => texts.statuses[STATUS_LOADING],
        UiStatus::Refreshing => texts.statuses[STATUS_REFRESHING],
        UiStatus::Ready => texts.statuses[STATUS_READY],
        UiStatus::Exhausted => texts.statuses[STATUS_EXHAUSTED],
        UiStatus::AccountRequired => texts.statuses[STATUS_ACCOUNT_REQUIRED],
        UiStatus::SubscriptionRequired => texts.statuses[STATUS_SUBSCRIPTION_REQUIRED],
        UiStatus::CodexMissing => texts.statuses[STATUS_CODEX_MISSING],
        UiStatus::Error => texts.statuses[STATUS_ERROR],
    }
}

fn status_color(status: UiStatus) -> COLORREF {
    match status {
        UiStatus::Error | UiStatus::Exhausted => rgb(255, 91, 110),
        UiStatus::AccountRequired | UiStatus::SubscriptionRequired | UiStatus::CodexMissing => {
            rgb(255, 181, 71)
        }
        UiStatus::Loading | UiStatus::Refreshing => rgb(101, 199, 242),
        UiStatus::Ready => rgb(52, 211, 153),
    }
}

fn ui_status(
    snapshot: Option<&UsageSnapshot>,
    error: Option<&(String, i64)>,
    querying: bool,
) -> UiStatus {
    if let Some((message, _)) = error {
        return classify_error(message);
    }
    if querying {
        return if snapshot.is_some() {
            UiStatus::Refreshing
        } else {
            UiStatus::Loading
        };
    }
    if snapshot.is_some_and(|value| value.used_percent >= 100 || value.limit_reached_type.is_some())
    {
        UiStatus::Exhausted
    } else if snapshot.is_some() {
        UiStatus::Ready
    } else {
        UiStatus::Loading
    }
}

fn classify_error(message: &str) -> UiStatus {
    let message = message.to_lowercase();
    if message.contains("не удалось запустить codex")
        || message.contains("codex: program not found")
    {
        UiStatus::CodexMissing
    } else if ["unauthorized", "not logged", "login", "auth", "401", "вход"]
        .iter()
        .any(|needle| message.contains(needle))
    {
        UiStatus::AccountRequired
    } else if [
        "subscription",
        "billing",
        "payment",
        "paid",
        "purchase",
        "403",
        "подпис",
        "оплат",
    ]
    .iter()
    .any(|needle| message.contains(needle))
    {
        UiStatus::SubscriptionRequired
    } else {
        UiStatus::Error
    }
}

fn tray_resource_for_state(state: &AppState) -> u16 {
    match ui_status(
        state.snapshot.as_ref(),
        state.last_error.as_ref(),
        state.querying,
    ) {
        UiStatus::Loading | UiStatus::Refreshing => LOADING_ICON_RESOURCE,
        UiStatus::AccountRequired | UiStatus::SubscriptionRequired => ACCOUNT_ICON_RESOURCE,
        UiStatus::CodexMissing => MISSING_ICON_RESOURCE,
        UiStatus::Error => ERROR_ICON_RESOURCE,
        UiStatus::Exhausted => status_icon_resource(0),
        UiStatus::Ready => state
            .snapshot
            .as_ref()
            .map(|snapshot| status_icon_resource(100_u32.saturating_sub(snapshot.used_percent)))
            .unwrap_or(LOADING_ICON_RESOURCE),
    }
}

fn credits_text(snapshot: &UsageSnapshot) -> String {
    if snapshot.unlimited_credits {
        "∞".into()
    } else {
        snapshot
            .credit_balance
            .clone()
            .unwrap_or_else(|| "0".into())
    }
}

fn format_window(minutes: Option<i64>, texts: &Texts) -> String {
    match minutes {
        Some(value) if value % 10_080 == 0 => {
            format!("{} {}", value / 10_080, texts.units[UNIT_WEEK])
        }
        Some(value) if value % 1_440 == 0 => {
            format!("{} {}", value / 1_440, texts.units[UNIT_DAY])
        }
        Some(value) if value % 60 == 0 => {
            format!("{} {}", value / 60, texts.units[UNIT_HOUR])
        }
        Some(value) => format!("{} {}", value, texts.units[UNIT_MINUTE]),
        None => texts.unknown_window.into(),
    }
}

fn format_datetime(timestamp: i64, texts: &Texts) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .map(|date| date.with_timezone(&Local).format("%d.%m %H:%M").to_string())
        .unwrap_or_else(|| texts.unknown.into())
}

unsafe fn create_system_font(hwnd: HWND, semibold: bool) -> HFONT {
    unsafe {
        let dpi = GetDpiForWindow(hwnd).max(96);
        let mut metrics = NONCLIENTMETRICSW {
            cbSize: size_of::<NONCLIENTMETRICSW>() as u32,
            ..Default::default()
        };
        if SystemParametersInfoForDpi(
            SPI_GETNONCLIENTMETRICS.0,
            metrics.cbSize,
            Some(&mut metrics as *mut NONCLIENTMETRICSW as *mut c_void),
            Default::default(),
            dpi,
        )
        .is_err()
        {
            metrics.lfMessageFont.lfHeight = -scale(hwnd, 12);
            copy_wide("Segoe UI", &mut metrics.lfMessageFont.lfFaceName);
        }
        if semibold {
            metrics.lfMessageFont.lfWeight = FW_SEMIBOLD.0 as i32;
        }
        CreateFontIndirectW(&metrics.lfMessageFont)
    }
}

unsafe fn select_font(hdc: windows::Win32::Graphics::Gdi::HDC, font: HFONT) {
    unsafe {
        let _ = SelectObject(hdc, HGDIOBJ(font.0));
    }
}

unsafe fn text_out(hdc: windows::Win32::Graphics::Gdi::HDC, x: i32, y: i32, text: &str) {
    unsafe {
        let text = wide(text);
        let _ = TextOutW(hdc, x, y, &text[..text.len() - 1]);
    }
}

unsafe fn fill_rect(hdc: windows::Win32::Graphics::Gdi::HDC, rect: &RECT, color: COLORREF) {
    unsafe {
        let brush = CreateSolidBrush(color);
        FillRect(hdc, rect, brush);
        let _ = DeleteObject(HGDIOBJ(brush.0));
    }
}

unsafe fn fill_rounded_rect(
    hdc: windows::Win32::Graphics::Gdi::HDC,
    rect: &RECT,
    color: COLORREF,
    background: COLORREF,
    radius: i32,
) {
    unsafe {
        let radius = radius
            .max(1)
            .min((rect.right - rect.left) / 2)
            .min((rect.bottom - rect.top) / 2);
        fill_rect(
            hdc,
            &RECT {
                left: rect.left + radius,
                right: rect.right - radius,
                ..*rect
            },
            color,
        );
        fill_rect(
            hdc,
            &RECT {
                top: rect.top + radius,
                bottom: rect.bottom - radius,
                ..*rect
            },
            color,
        );

        const SAMPLES: i32 = 4;
        for corner_y in 0..radius {
            for corner_x in 0..radius {
                let mut coverage = 0;
                for sample_y in 0..SAMPLES {
                    for sample_x in 0..SAMPLES {
                        let x = corner_x as f32 + (sample_x as f32 + 0.5) / SAMPLES as f32;
                        let y = corner_y as f32 + (sample_y as f32 + 0.5) / SAMPLES as f32;
                        let dx = radius as f32 - x;
                        let dy = radius as f32 - y;
                        coverage += (dx * dx + dy * dy <= (radius * radius) as f32) as u32;
                    }
                }
                let blended = blend_color(
                    background,
                    color,
                    coverage as f32 / (SAMPLES * SAMPLES) as f32,
                );
                for (x, y) in [
                    (rect.left + corner_x, rect.top + corner_y),
                    (rect.right - 1 - corner_x, rect.top + corner_y),
                    (rect.left + corner_x, rect.bottom - 1 - corner_y),
                    (rect.right - 1 - corner_x, rect.bottom - 1 - corner_y),
                ] {
                    let _ = SetPixelV(hdc, x, y, blended);
                }
            }
        }
    }
}

fn blend_color(background: COLORREF, foreground: COLORREF, alpha: f32) -> COLORREF {
    let channel = |shift: u32| {
        let background = ((background.0 >> shift) & 0xff) as f32;
        let foreground = ((foreground.0 >> shift) & 0xff) as f32;
        (background + (foreground - background) * alpha).round() as u8
    };
    rgb(channel(0), channel(8), channel(16))
}

fn remaining_color(percent: u32) -> COLORREF {
    if percent <= 10 {
        rgb(255, 91, 110)
    } else if percent <= 30 {
        rgb(255, 181, 71)
    } else {
        windows_accent_color().unwrap_or_else(|| rgb(0, 120, 212))
    }
}

fn windows_accent_color() -> Option<COLORREF> {
    unsafe {
        let mut argb = 0_u32;
        let mut opaque = BOOL::default();
        DwmGetColorizationColor(&mut argb, &mut opaque).ok()?;
        Some(rgb(
            ((argb >> 16) & 0xff) as u8,
            ((argb >> 8) & 0xff) as u8,
            (argb & 0xff) as u8,
        ))
    }
}

fn status_icon_resource(percent: u32) -> u16 {
    let (resource, _) = STATUS_ICON_RESOURCES
        .iter()
        .map(|&(level, resource)| (resource, level.abs_diff(percent)))
        .min_by_key(|&(_, distance)| distance)
        .expect("status icon states are defined");
    resource
}

unsafe fn load_status_icon(hwnd: HWND, resource: u16) -> WinResult<HICON> {
    unsafe {
        let instance = GetModuleHandleW(None)?;
        let dpi = GetDpiForWindow(hwnd).max(96);
        let size = GetSystemMetricsForDpi(SM_CXSMICON, dpi);
        let handle = LoadImageW(
            Some(HINSTANCE(instance.0)),
            resource_name(resource),
            IMAGE_ICON,
            size,
            size,
            LR_DEFAULTCOLOR,
        )?;
        Ok(HICON(handle.0))
    }
}

const fn resource_name(resource: u16) -> PCWSTR {
    PCWSTR(resource as usize as *const u16)
}

unsafe fn scale(hwnd: HWND, logical_pixels: i32) -> i32 {
    unsafe {
        let dpi = GetDpiForWindow(hwnd).max(96) as i64;
        ((logical_pixels as i64 * dpi + 48) / 96) as i32
    }
}

const fn rgb(red: u8, green: u8, blue: u8) -> COLORREF {
    COLORREF(red as u32 | ((green as u32) << 8) | ((blue as u32) << 16))
}

fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(Some(0)).collect()
}

fn copy_wide<const N: usize>(value: &str, target: &mut [u16; N]) {
    target.fill(0);
    for (destination, source) in target
        .iter_mut()
        .take(N.saturating_sub(1))
        .zip(value.encode_utf16())
    {
        *destination = source;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_common_windows() {
        assert_eq!(format_window(Some(300), &RUSSIAN_TEXTS), "5 ч");
        assert_eq!(format_window(Some(10_080), &ENGLISH_TEXTS), "1 wk");
    }

    #[test]
    fn chooses_nearest_remaining_quota_icon() {
        assert_eq!(status_icon_resource(0), 100);
        assert_eq!(status_icon_resource(3), 101);
        assert_eq!(status_icon_resource(24), 102);
        assert_eq!(status_icon_resource(97), 105);
        assert_eq!(status_icon_resource(100), 106);
    }

    #[test]
    fn classifies_account_and_subscription_failures() {
        assert_eq!(
            classify_error("Unauthorized: login required"),
            UiStatus::AccountRequired
        );
        assert_eq!(
            classify_error("Subscription payment required"),
            UiStatus::SubscriptionRequired
        );
        assert_eq!(
            classify_error("не удалось запустить codex: файл не найден"),
            UiStatus::CodexMissing
        );
        assert_eq!(classify_error("app-server crashed"), UiStatus::Error);
    }

    #[test]
    fn quotes_autostart_executable_path() {
        assert_eq!(
            quote_executable(Path::new(r"C:\Portable Apps\Codex Tray\codex-tray.exe")),
            r#""C:\Portable Apps\Codex Tray\codex-tray.exe""#
        );
    }

    #[test]
    fn finds_executable_directory() {
        assert_eq!(
            executable_directory(Path::new(r"C:\Portable Apps\Codex Tray\codex-tray.exe"))
                .expect("directory"),
            Path::new(r"C:\Portable Apps\Codex Tray")
        );
        assert!(executable_directory(Path::new("codex-tray.exe")).is_none());
    }

    #[test]
    fn maps_supported_system_locales() {
        assert_eq!(language_from_locale_name("ru-RU"), Language::Russian);
        assert_eq!(
            language_from_locale_name("zh_Hans_CN"),
            Language::SimplifiedChinese
        );
        assert_eq!(language_from_locale_name("ko-KR"), Language::Korean);
        assert_eq!(language_from_locale_name("sv-SE"), Language::English);
    }

    #[test]
    fn exposes_supported_languages_in_menu_order() {
        assert_eq!(
            LANGUAGES
                .iter()
                .map(|language| (language.code(), language.native_name()))
                .collect::<Vec<_>>(),
            [
                ("en", "English"),
                ("es", "Español"),
                ("fr", "Français"),
                ("pt", "Português"),
                ("de", "Deutsch"),
                ("it", "Italiano"),
                ("ru", "Русский"),
                ("zh-CN", "简体中文"),
                ("hi", "हिन्दी"),
                ("ar", "العربية"),
                ("ja", "日本語"),
                ("ko", "한국어"),
            ]
        );
    }

    #[test]
    fn parses_and_serializes_portable_settings() {
        let settings = Settings::parse(
            PathBuf::from(r"C:\Apps\Codex Tray\codex-tray.json"),
            r#"{"language":"ja","start_with_windows":true}"#,
        )
        .expect("settings");
        assert_eq!(
            settings.language,
            LanguagePreference::Selected(Language::Japanese)
        );
        assert!(settings.start_with_windows);
        let serialized = settings.serialize().expect("serialized settings");
        assert!(serialized.contains(r#""language": "ja""#));
        assert!(serialized.contains(r#""start_with_windows": true"#));
        assert!(serialized.ends_with('\n'));
    }

    #[test]
    fn rejects_unsupported_configured_language() {
        let error = Settings::parse(
            PathBuf::from("codex-tray.json"),
            r#"{"language":"sv","start_with_windows":false}"#,
        );
        assert!(matches!(error, Err(SettingsError::Invalid)));
    }
}
