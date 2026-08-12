# Citrix VDI Launcher

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · العربية · [日本語](README.ja.md) · [한국어](README.ko.md)

---

<div dir="rtl">

عميل desktop وCLI للاتصال بـ Citrix Gateway وStoreFront VDI دون متصفح.

## نظرة عامة والميزات

ينفذ authentication وOTP/TOTP واكتشاف VDI وتنزيل ICA وتشغيل Citrix Workspace. تشترك GUI وCLI في نواة Rust؛ وتحمي الأسرار عبر Windows DPAPI أو macOS Keychain أو Linux Secret Service. المشروع مستقل وغير تابع لـ Citrix Systems, Inc.

## المتطلبات والتثبيت

يلزم Citrix Workspace وGateway/StoreFront متوافق، مع Windows x86-64 أو macOS Intel/Apple Silicon أو Linux x86-64 مع Wayland/X11. نزّل Windows ZIP أو Linux DEB/RPM x86-64 أو macOS universal ZIP من [GitHub Releases](https://github.com/psimonov/citrix-launcher/releases/latest). حزم Linux ARM64 غير منشورة بعد. يعرض GitHub قيمة SHA-256 لكل asset.

## بدء سريع

</div>

```text
citrix-vdi-cli config set --storefront https://gateway.example/ --vdi MY-DESKTOP --username user
citrix-vdi-cli connect
citrix-vdi-cli connect --otp 123456
```

<div dir="rtl">

تتوفر أيضاً `config show` و`config path` و`detect-citrix` وحفظ `--password`/`--totp-secret`. تبقى الأسرار في credential store للنظام.

## البناء

</div>

```text
cargo fmt --all -- --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo build --release --bins
```

<div dir="rtl">

ينشر GitHub Actions الإصدارات من SemVer tags.

## الأمان والمساهمة والترخيص

لا تنشر credentials أو OTP/TOTP أو ICA أو cookies أو CSRF tokens أو logs. راجع [SECURITY.md](SECURITY.md) و[CONTRIBUTING.md](CONTRIBUTING.md). [رخصة MIT](LICENSE).

</div>
