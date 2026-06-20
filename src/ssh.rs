use ssh2::Session;
use std::io::Read;
use std::net::TcpStream;
use anyhow::{Context, Result, bail};
use crate::recipe::LaziRecipe;

// helper func for clean SSH execution.
pub fn ssh_cmd(sess: &Session, cmd: &str) -> Result<()> {
    println!("Executing: {}", cmd);
    let mut channel = sess.channel_session().context("Failed to open SSH Channel")?;
    channel.exec(cmd).context("Failed to execute command")?;

    let mut output = String::new();
    channel.read_to_string(&mut output).context("Failed to read output")?;

    let trimmed = output.trim();
    if !trimmed.is_empty() {
        println!("   ↳ {}", trimmed.replace("\n", "\n   ↳ "));
    }

    // Clean up
    channel.wait_close().context("Failed to close channel gracefully")?;
    Ok(())
}

pub fn config_vm(recipe: &LaziRecipe, user_password: &str) -> Result<()> {
    let target = "127.0.0.1:2222";

    println!("\n====================");
    println!("REMOTE CONFIGURATION");
    println!("====================");
    println!("Connecting to {}...", target);

    let tcp = TcpStream::connect(target).context("Failed to connect via TCP. Is the VM online and port-forwarded?")?;
    let mut sess = Session::new().unwrap();
    sess.set_tcp_stream(tcp);
    sess.handshake().context("SSH handshake failed")?;

    sess.userauth_password("kali", "kali").context("SSH auth failed")?;
    if !sess.authenticated() {
        bail!("SSH Authentication failed. Check credentials.");
    }
    println!("Authenticated successfully.\n");

    // YAML Integration: User Config
    if let Some(user) = &recipe.user {
        println!("Setting up user '{}'...", user.username);
        let add_user = format!("echo kali | sudo -S useradd -m -s {} {}", user.shell, user.username);
        let set_pass = format!("echo kali | sudo -S bash -c \"echo '{}:{}' | chpasswd\"", user.username, user_password);
        let add_sudo = format!("echo kali | sudo -S usermod -aG sudo {}", user.username);

        let _ = ssh_cmd(&sess, &add_user); // ignore erros incase user already exists.
        ssh_cmd(&sess, &set_pass)?;
        ssh_cmd(&sess, &add_sudo)?;
    }

    // YAML Integration: APT Packages
    if let Some(packages) = &recipe.packages {
        if let Some(apt_pkgs) =  &packages.apt {
            println!("Installing APT Packages...");
            ssh_cmd(&sess, "echo kali | sudo -S apt-get update")?;

            let pkg_list = apt_pkgs.join(" "); // joins ["nmap", "ffuf"] to "nmap ffuf" 
            let installed_cmd = format!("echo kali | sudo -S DEBIAN_FRONTEND=noninteractive apt-get install -y {}", pkg_list);
            ssh_cmd(&sess, &installed_cmd)?;
        }

        // YAML Integration: PIPX Packages
        if let Some(pipx_pkgs) = &packages.pipx {
            println!("Installing PIPX Packages...");
            ssh_cmd(&sess, "echo kali | sudo -S DEBIAN_FRONTEND=noninteractive apt-get install -y pipx")?;
            for pkg in pipx_pkgs {
                let pipx_cmd = format!("pipx install {}", pkg);
                ssh_cmd(&sess, &pipx_cmd)?;
            }
        }
    }

    // YAML Integration: Scripts
    if let Some(scripts) = &recipe.scripts {
        println!("Execution Custom Scripts...");
        for script in scripts {
            ssh_cmd(&sess, script)?;
        }
    }

    // YAML Integration

    if let Some(files) = &recipe.files { 
        println!("Writing Custom Files...");

        let home_dir = if let Some(user) = &recipe.user {
            format!("/home/{}", user.username)
        } else {
            "/root".to_string()
        };

        for file in files {
            let file_path = file.path.replace("{{HOME}}", &home_dir);

            if let Some(content) = &file.content {
                println!("   ↳ Creating {}...", file_path);

                // using HereDOc for multi-line strings
                let wcmd = format!(
                    "echo kali | sudo -S bash -c \"cat << 'EOF' > {}\n{}\nEOF\"",
                    file_path,
                    content
                );

                ssh_cmd(&sess, &wcmd)?;

                // Modifying file ownership
                if let Some(user) = &recipe.user {
                    let chcmd = format!("echo kali | sudo -S chown {}:{} {}", user.username, user.username, file_path);
                    let _ = ssh_cmd(&sess, &chcmd);
                }
            }

        }
    }

    println!("\nSSH Provisioning Complete.");
    Ok(())
}


