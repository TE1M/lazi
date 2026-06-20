use crate::recipe::LaziRecipe;
use std::process::Command as ProcessCommand;
use std::time::Duration;
use std::thread;
use std::io::Write;
use anyhow::{Context, Result, bail};

pub fn vbox_cmd(args: &[&str], fatal: bool) -> Result<()> {
    println!("Executing: VBoxManage {}", args.join(" "));

    let output = ProcessCommand::new("VBoxManage")
        .args(args)
        .output()
        .context("Failed to execute VBoxManage. Is VirtualBox Installed?")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        if fatal {
            // safely pass error up chain instead of crashing.
            bail!("Fail VBoxManage Error: {}", err.trim());
        } else {
            eprint!("VBoxManage Note: {}", err.trim());
        }
    }
    Ok(())
}

pub fn vbox_auto(recipe: &LaziRecipe) -> Result<(), anyhow::Error> {
    let base_vm = "kali-base"; // TO-DO Change
    let new_vm = &recipe.name;

    if base_vm == new_vm {
        bail!("Aborting: Your lazi-recipe.yaml 'name' cannot be the same name as the golden image ('{}'), Lazi would delete the ancestor image!", base_vm);
    }

    println!("\n====================");
    println!("HYPERVISOR AUTOMATION: {}", new_vm);
    println!("====================");

    // Env cleanup
    println!("Cleaning up environment...");
    let _ = vbox_cmd(&["controlvm", new_vm, "poweroff"], false);
    thread::sleep(Duration::from_secs(2));
    let _ = vbox_cmd(&["unregistervm", new_vm, "--delete"], false);

    // Clone base
    println!("Cloning '{}' into '{}'...", base_vm, new_vm);
    vbox_cmd(&["clonevm", base_vm, "--name", new_vm, "--register"], true)?;

    // YAML integration
    println!("Applying hardware configs ({} CPUs, {}MB RAM)...", recipe.vm.cpus, recipe.vm.ram_mb);
    vbox_cmd(&["modifyvm", new_vm, "--cpus", &recipe.vm.cpus.to_string(), "--memory", &recipe.vm.ram_mb.to_string()], true)?;

    println!("Configuring Network Adapter...");
    vbox_cmd(&["modifyvm", new_vm, "--nic1", "nat"], true)?;
    let _ = vbox_cmd(&["modifyvm", new_vm, "--natpf1", "delete", "lazissh"], false);
    vbox_cmd(&["modifyvm", new_vm, "--natpf1", "lazissh,tcp,127.0.0.1,2222,,22"], true)?;

    println!("Booting '{}' in Headless mode...", new_vm);
    vbox_cmd(&["startvm", new_vm, "--type", "headless"], true)?;

    
    // Smart polling for race condition issue
    //
    println!("Waiting for VM and Guest Addition to boot...");
    thread::sleep(Duration::from_secs(30));

    let mut ssh_enabled = false;
    println!("Polling VirtualBox");
    let _ = std::io::stdout().flush();

    for _attempt in 1..=20 { // Try 20 times.
        let output = ProcessCommand::new("VBoxManage")
            .args(&[
                "guestcontrol", new_vm, "run",
                "--exe", "/bin/bash",
                "--username", "kali",
                "--password", "kali",
                "--", "-c", "echo kali | sudo -S systemctl start ssh"
            ])
            .output()
            .context("Failed to execute VBoxManage")?;
                
        if output.status.success() {
            println!("\nGuest Additions are ready and SSH is enabled.");
            ssh_enabled = true;
            break;
        }

        println!(".");
        let _ = std::io::stdout().flush();
        thread::sleep(Duration::from_secs(5));
    }

    if !ssh_enabled {
        println!(); // clear dots
        bail!("Guest Additions never became ready. Is the golden image stuck booting?");
    }

    println!("VM is live and ready for SSH on port 2222.");
    Ok(())
}

