use std::process::Command;
use anyhow::Result;
use notify_rust::Notification;

pub fn send_notification(title: &str, message: &str, success: bool) -> Result<()> {
    if cfg!(target_os = "windows") {
        let icon = if success { "✅" } else { "❌" };
        let script = format!(
            "New-BurntToastNotification -Text '{}', '{} {}' -Silent",
            title, icon, message
        );
        Command::new("powershell")
            .arg("-Command")
            .arg(script)
            .output()?;
    } else {
        // Use notify-rust for Unix-like systems (Linux and macOS)
        let mut notification = Notification::new();
        
        notification
            .summary(title)
            .body(&format!("{} {}", if success { "✅" } else { "❌" }, message))
            .timeout(5000); // 5 seconds

        // Set appropriate icon based on the platform
        if cfg!(target_os = "macos") {
            notification.sound_name("default");
        } else {
            notification.icon(if success { "dialog-ok" } else { "dialog-error" });
        }

        notification.show()?;
    }
    Ok(())
} 