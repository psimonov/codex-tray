# Codex Tray

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · हिन्दी · [العربية](README.ar.md) · [日本語](README.ja.md) · [한국어](README.ko.md)

---

Codex की बची हुई उपयोग सीमा देखने के लिए एक नेटिव Windows सिस्टम-ट्रे संकेतक।

## परिचय

Codex Tray आपको Codex ऐप या CLI को सामने रखे बिना मौजूदा Codex कोटा देखने देता है। यह Windows सिस्टम ट्रे में एक छोटे ऐप की तरह चलता है, वर्तमान उपयोगकर्ता के प्रमाणित Codex CLI सत्र का उपयोग करता है और आइकन पर पॉइंटर ले जाने पर एक संक्षिप्त कोटा पैनल दिखाता है।

ऐप केवल स्थानीय रूप से स्थापित `codex app-server` से संवाद करता है। यह API कुंजी नहीं माँगता और `~/.codex/auth.json` को सीधे पढ़ता या कॉपी नहीं करता।

## विशेषताएँ

- `account/rateLimits/updated` सर्वर सूचनाओं के माध्यम से कोटा का लाइव अपडेट और पुराने डेटा के लिए hover पर freshness जाँच।
- स्थिर `लेबल: मान` पंक्तियों वाला संक्षिप्त DPI-aware पैनल।
- Windows के लाइट/डार्क थीम, एक्सेंट रंग और पारदर्शिता का समर्थन।
- कोटा स्तरों और त्रुटि स्थितियों के लिए पिक्सेल-अलाइन ट्रे आइकन।
- पॉइंटर ले जाने पर पैनल आइकन वाले monitor पर दिखता है और हटाने पर छिप जाता है।
- 12 भाषाओं के अंतर्निहित अनुवाद, जिनमें सिस्टम भाषा डिफ़ॉल्ट रूप से चुनी जाती है।
- माँग पर अपडेट, executable folder खोलने, Windows के साथ शुरू करने के नियंत्रण और स्पष्ट बंद करने की क्रिया वाला संदर्भ मेनू।
- executable के पास संग्रहीत पोर्टेबल भाषा और startup सेटिंग्स।
- ट्रे आइकन पर कोई सिस्टम टूलटिप नहीं।
- लोडिंग, दोबारा कनेक्ट होने, प्रमाणीकरण, सदस्यता, CLI न मिलने, कोटा समाप्त होने और app-server त्रुटि के अलग-अलग संकेत।

## आवश्यकताएँ

- Windows 11 x86-64।
- `PATH` में उपलब्ध [Codex CLI](https://learn.chatgpt.com/docs/codex/cli)।
- `codex login` से बनाया गया प्रमाणित Codex CLI सत्र।

Codex Tray में अभी केवल नेटिव Windows backend लागू है। Linux, macOS और Windows ARM64 के platform backend लागू और परीक्षण किए जाने तक उनके artefact प्रकाशित नहीं किए जाते।

## स्थापना

1. [नवीनतम GitHub Release](https://github.com/psimonov/codex-tray/releases/latest) खोलें।
2. `codex-tray-<version>-windows-x86_64.exe` और उसकी `.sha256` फ़ाइल डाउनलोड करें।
3. SHA-256 checksum सत्यापित करें।
4. executable को किसी स्थायी और लिखने योग्य फ़ोल्डर में ले जाकर चलाएँ।

PowerShell में checksum जाँचने का उदाहरण:

```powershell
Get-FileHash .\codex-tray-0.4.1-windows-x86_64.exe -Algorithm SHA256
```

Installer की आवश्यकता नहीं है। Release एक ही portable executable है; `codex` कमांड बाहरी runtime आवश्यकता बनी रहती है।

## तुरंत शुरू करें

```powershell
codex login
.\codex-tray-0.4.1-windows-x86_64.exe
```

ऐप छिपी हुई अवस्था में शुरू होता है और Windows सिस्टम ट्रे में अपना आइकन जोड़ता है।

## उपयोग

- उसी monitor पर कोटा पैनल दिखाने के लिए ट्रे आइकन पर पॉइंटर ले जाएँ।
- पैनल छिपाने के लिए पॉइंटर आइकन से दूर ले जाएँ।
- पैनल छिपाकर संदर्भ मेनू खोलने के लिए आइकन पर दायाँ क्लिक करें।
- **भाषा** उपमेनू खोलें और **सिस्टम भाषा** या कोई विशिष्ट भाषा चुनें। बदलाव तुरंत लागू होता है।
- मौजूदा app-server connection पर `account/read` और `account/rateLimits/read` को तुरंत दोहराने के लिए **अभी अपडेट करें** चुनें।
- चल रहे executable वाली directory खोलने के लिए **ऐप का folder खोलें** चुनें।
- वर्तमान executable path को उपयोगकर्ता के `Run` key में दर्ज करने या हटाने के लिए **Windows के साथ शुरू करें** को टॉगल करें।
- Codex Tray और उसके app-server child process को रोकने के लिए **बंद करें** चुनें।

कोटा अपडेट `codex app-server` के साथ एक स्थायी connection से आते हैं। Codex Tray शुरुआत में account और limits को एक बार पढ़ता है, बाद की आंशिक सूचनाओं को सुरक्षित रखकर recursively मिलाता है और app-server के अनपेक्षित रूप से बंद होने पर फिर से जुड़ता है। स्पष्ट refresh दोनों requests दोहराता है। यदि पैनल दिखाते समय snapshot कम से कम 30 सेकंड पुराना है, तो Codex Tray उसे `account/rateLimits/read` से एक बार reconcile करता है; background में periodic polling नहीं होता।

## कॉन्फ़िगरेशन

पहली बार शुरू होने पर Codex Tray executable के पास `codex-tray.json` बनाता है। इसमें चुनी हुई भाषा और Windows startup सेटिंग संग्रहीत होती हैं:

```json
{
  "language": "system",
  "start_with_windows": false
}
```

`language` में `system`, `en`, `es`, `fr`, `pt`, `de`, `it`, `ru`, `zh-CN`, `hi`, `ar`, `ja` या `ko` स्वीकार किए जाते हैं। configuration file सेटिंग्स का स्रोत है। उपयोगकर्ता की Windows `Run` entry को `start_with_windows` से synchronize किया जाता है और उसमें हमेशा चल रहे executable का dynamically detected path होता है। configuration पहली बार बनाते समय मौजूदा startup entry import की जाती है।

## स्रोत से build करें

Repository आवश्यक Rust toolchain को pin करता है।

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

बना हुआ executable `target\release\codex-tray.exe` है।

## परीक्षण

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## रिलीज़

Version tag का प्रारूप `vMAJOR.MINOR.PATCH` है। GitHub Actions tag को `Cargo.toml` के version से मिलाता है, project checks चलाता है, Windows x86-64 executable बनाता है और executable तथा उसका SHA-256 checksum एक ही GitHub Release में प्रकाशित करता है।

Project अभी केवल Windows का समर्थन करता है, इसलिए सिर्फ Windows x86-64 artefact प्रकाशित होते हैं। यह एक स्पष्ट platform निर्णय है, अप्रमाणित cross-platform support का दावा नहीं।

## सुरक्षा

समर्थित versions और vulnerability की निजी रिपोर्टिंग के तरीके के लिए [SECURITY.md](SECURITY.md) देखें। Public issue में vulnerability प्रकट न करें।

## योगदान

Development workflow और commit आवश्यकताओं के लिए [CONTRIBUTING.md](CONTRIBUTING.md) देखें।

## लाइसेंस

Codex Tray [MIT License](LICENSE) के अंतर्गत उपलब्ध है।

## Protocol reference

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)
