use amml_sdk::{HookHandle, SdkError};

use crate::services::PingRuntime;
use crate::traits::GameHost;
use crate::types::HookSpec;

// installs the ping hooks: every required entry must resolve before any hook goes in; a missing optional
// entry or a refused optional hook turns off only its own feature, with one log line. A refused required hook
// removes every hook and fails. Two hooks whose targets are one address are installed
// once: the first hook serves both
pub struct HookInstaller;

impl HookInstaller {
    // the features turned off by a missing optional entry, each once, in spec order
    pub fn install<H: GameHost>(
        host: &H,
        runtime: &PingRuntime,
        specs: &[HookSpec],
    ) -> Result<Vec<&'static str>, SdkError> {
        let mut targets: Vec<(u64, &HookSpec)> = Vec::with_capacity(specs.len());
        let mut off: Vec<&'static str> = Vec::new();
        let mut warned: Vec<String> = Vec::new();
        for spec in specs {
            match Self::target(host, spec) {
                Ok(target) if targets.iter().any(|(kept, _)| *kept == target) => {}
                Ok(target) => targets.push((target, spec)),
                Err(error) if spec.required => return Err(error),
                Err(error) => {
                    let line = format!("{error}; {} is off", spec.feature);
                    Self::turn_off(host, &mut off, &mut warned, spec.feature, line);
                }
            }
        }
        for (target, spec) in targets {
            let kept = match host.hook(target, spec) {
                Ok(hook) => Self::keep(host, runtime, hook),
                Err(error) => Err(error),
            };
            match kept {
                Ok(()) => {}
                Err(error) if spec.required => {
                    Self::remove_all(host, runtime);
                    return Err(error);
                }
                Err(error) => {
                    let line = format!("{} could not be hooked ({error}); {} is off", spec.entry, spec.feature);
                    Self::turn_off(host, &mut off, &mut warned, spec.feature, line);
                }
            }
        }
        Ok(off)
    }

    pub fn remove_all<H: GameHost>(host: &H, runtime: &PingRuntime) {
        for hook in runtime.take_hooks() {
            if let Err(error) = host.unhook(hook) {
                host.warn(&format!("unhook failed: {error}"));
            }
        }
    }

    // the function address `spec.entry` names, or the code pointer in that slot of the `spec.table` vtable. A
    // required entry's error is returned as is; an optional one's names the entry that failed
    fn target<H: GameHost>(host: &H, spec: &HookSpec) -> Result<u64, SdkError> {
        let resolve = |id: &str| {
            host.resolve(id).map_err(|error| {
                if spec.required { error } else { SdkError::Mod(format!("{id} did not resolve ({error})")) }
            })
        };
        match spec.table {
            None => resolve(spec.entry),
            Some(table) => {
                let table = resolve(table)?;
                let slot = resolve(spec.entry)?;
                host.table_slot(table, slot)
                    .map_err(|error| SdkError::Mod(format!("{} in its table did not resolve ({error})", spec.entry)))
            }
        }
    }

    // notes `feature` as off and logs `line`, each once
    fn turn_off<H: GameHost>(
        host: &H,
        off: &mut Vec<&'static str>,
        warned: &mut Vec<String>,
        feature: &'static str,
        line: String,
    ) {
        if !warned.contains(&line) {
            host.warn(&line);
            warned.push(line);
        }
        if !off.contains(&feature) {
            off.push(feature);
        }
    }

    // a hook the runtime cannot keep is removed again, so no installed hook is left without a handle
    fn keep<H: GameHost>(host: &H, runtime: &PingRuntime, hook: HookHandle) -> Result<(), SdkError> {
        if runtime.keep_hook(hook) {
            return Ok(());
        }
        let _ = host.unhook(hook);
        Err(SdkError::Mod(format!("more than {} hooks", PingRuntime::HOOK_CAPACITY)))
    }
}
