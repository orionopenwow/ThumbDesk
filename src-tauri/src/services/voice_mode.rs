use std::io::{BufRead, BufReader};
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;

use serde::{Deserialize, Serialize};

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x0800_0000;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceCommandRule {
    pub id: String,
    pub phrase: String,
    pub command: String,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VoiceModeState {
    pub enabled: bool,
    pub listening: bool,
    pub last_heard: Option<String>,
    pub last_executed: Option<String>,
    pub status_message: String,
    pub rules: Vec<VoiceCommandRule>,
}

impl Default for VoiceModeState {
    fn default() -> Self {
        Self {
            enabled: false,
            listening: false,
            last_heard: None,
            last_executed: None,
            status_message: "Voice mode off (offline Windows Speech)".to_string(),
            rules: vec![
                VoiceCommandRule {
                    id: "open-terminal".into(),
                    phrase: "open terminal".into(),
                    command: "wt.exe".into(),
                    enabled: true,
                },
                VoiceCommandRule {
                    id: "open-notepad".into(),
                    phrase: "open notepad".into(),
                    command: "notepad.exe".into(),
                    enabled: true,
                },
                VoiceCommandRule {
                    id: "open-calc".into(),
                    phrase: "open calculator".into(),
                    command: "calc.exe".into(),
                    enabled: true,
                },
                VoiceCommandRule {
                    id: "lock-pc".into(),
                    phrase: "lock computer".into(),
                    command: "rundll32.exe user32.dll,LockWorkStation".into(),
                    enabled: true,
                },
            ],
        }
    }
}

/// Executes any user-configured Windows command / PowerShell / .bat / .exe script silently.
pub fn execute_command_script(script: &str) -> Result<(), String> {
    let trimmed = script.trim();
    if trimmed.is_empty() {
        return Err("Command script is empty".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        Command::new("cmd")
            .args(["/C", trimmed])
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| format!("Failed to launch command: {}", e))?;
    }
    #[cfg(not(target_os = "windows"))]
    {
        Command::new("sh")
            .args(["-c", trimmed])
            .spawn()
            .map_err(|e| format!("Failed to launch command: {}", e))?;
    }
    Ok(())
}

/// Matches a recognized speech phrase against enabled rules (case-insensitive).
pub fn match_voice_rule<'a>(
    phrase: &str,
    rules: &'a [VoiceCommandRule],
) -> Option<&'a VoiceCommandRule> {
    let target = phrase.trim().to_ascii_lowercase();
    rules
        .iter()
        .find(|r| r.enabled && r.phrase.trim().to_ascii_lowercase() == target)
}

struct VoiceInner {
    state: VoiceModeState,
    child: Option<Child>,
    generation: u64,
}

#[derive(Clone)]
pub struct VoiceModeService {
    inner: Arc<Mutex<VoiceInner>>,
}

impl Default for VoiceModeService {
    fn default() -> Self {
        Self::new()
    }
}

impl VoiceModeService {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(VoiceInner {
                state: VoiceModeState::default(),
                child: None,
                generation: 0,
            })),
        }
    }

    pub fn state(&self) -> VoiceModeState {
        self.inner.lock().unwrap().state.clone()
    }

    pub fn set_rules(&self, rules: Vec<VoiceCommandRule>) -> VoiceModeState {
        let was_enabled = {
            let mut guard = self.inner.lock().unwrap();
            guard.state.rules = rules;
            guard.state.enabled
        };
        if was_enabled {
            self.set_enabled(false);
            self.set_enabled(true)
        } else {
            self.state()
        }
    }

    pub fn toggle(&self) -> VoiceModeState {
        let current = self.inner.lock().unwrap().state.enabled;
        self.set_enabled(!current)
    }

    pub fn set_enabled(&self, enabled: bool) -> VoiceModeState {
        let (gen, phrases) = {
            let mut guard = self.inner.lock().unwrap();
            if let Some(mut c) = guard.child.take() {
                let _ = c.kill();
            }
            guard.generation = guard.generation.wrapping_add(1);
            guard.state.enabled = enabled;
            if !enabled {
                guard.state.listening = false;
                guard.state.status_message = "Voice mode off (offline Windows Speech)".to_string();
                return guard.state.clone();
            }
            let phrases: Vec<String> = guard
                .state
                .rules
                .iter()
                .filter(|r| r.enabled && !r.phrase.trim().is_empty())
                .map(|r| r.phrase.trim().replace('\'', "''"))
                .collect();
            if phrases.is_empty() {
                guard.state.listening = false;
                guard.state.status_message = "Add at least one voice command rule.".to_string();
                return guard.state.clone();
            }
            guard.state.listening = true;
            guard.state.status_message =
                format!("Listening offline for {} voice commands…", phrases.len());
            (guard.generation, phrases)
        };

        let choices_ps = phrases
            .iter()
            .map(|p| format!("'{}'", p))
            .collect::<Vec<_>>()
            .join(",");

        let ps_script = format!(
            "Add-Type -AssemblyName System.Speech; \
             $rec = New-Object System.Speech.Recognition.SpeechRecognitionEngine; \
             $choices = New-Object System.Speech.Recognition.Choices(@({choices})); \
             $gb = New-Object System.Speech.Recognition.GrammarBuilder; \
             $gb.Append($choices); \
             $rec.LoadGrammar((New-Object System.Speech.Recognition.Grammar($gb))); \
             $rec.SetInputToDefaultAudioDevice(); \
             while ($true) {{ \
               $res = $rec.Recognize([TimeSpan]::FromSeconds(2)); \
               if ($res -and $res.Confidence -ge 0.60) {{ \
                 [Console]::Out.WriteLine($res.Text); \
                 [Console]::Out.Flush(); \
               }} \
             }}",
            choices = choices_ps
        );

        let mut cmd = Command::new("powershell");
        cmd.args(["-NoProfile", "-Command", &ps_script])
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(target_os = "windows")]
        {
            cmd.creation_flags(CREATE_NO_WINDOW);
        }
        let spawned = cmd.spawn();

        match spawned {
            Ok(mut child) => {
                let stdout = child.stdout.take();
                {
                    let mut guard = self.inner.lock().unwrap();
                    if guard.generation != gen {
                        let _ = child.kill();
                        return guard.state.clone();
                    }
                    guard.child = Some(child);
                }
                if let Some(out) = stdout {
                    let svc = self.clone();
                    thread::Builder::new()
                        .name("thumbdesk-voice-listener".into())
                        .spawn(move || {
                            let reader = BufReader::new(out);
                            for line in reader.lines().map_while(Result::ok) {
                                let phrase = line.trim().to_string();
                                if phrase.is_empty() {
                                    continue;
                                }
                                let cmd_to_run = {
                                    let mut guard = svc.inner.lock().unwrap();
                                    if guard.generation != gen || !guard.state.enabled {
                                        break;
                                    }
                                    guard.state.last_heard = Some(phrase.clone());
                                    if let Some(rule) = match_voice_rule(&phrase, &guard.state.rules)
                                    {
                                        let cmd = rule.command.clone();
                                        guard.state.last_executed = Some(cmd.clone());
                                        guard.state.status_message =
                                            format!("Heard \"{}\" → {}", phrase, cmd);
                                        Some(cmd)
                                    } else {
                                        None
                                    }
                                };
                                if let Some(cmd) = cmd_to_run {
                                    let _ = execute_command_script(&cmd);
                                }
                            }
                        })
                        .ok();
                }
            }
            Err(e) => {
                let mut guard = self.inner.lock().unwrap();
                guard.state.listening = false;
                guard.state.status_message = format!("Microphone / Speech unavailable: {}", e);
            }
        }

        self.state()
    }
}
