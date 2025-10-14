//! code for the client daemon

use nix;
use std::collections::HashMap;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::net::TcpStream;
mod dbus;
use zbus::blocking::Connection;

use crate::client::dbus::NotificationsProxyBlocking;
use crate::notifications::NotificationDetails;
use crate::protocol;

pub fn main(connect: String) -> anyhow::Result<()> {
    let hostname = nix::unistd::gethostname()?;
    let hostname = hostname.to_string_lossy();
    let uid = nix::unistd::getuid();
    let user = nix::unistd::User::from_uid(nix::unistd::getuid())?.map_or(uid.to_string(), |u| u.name);

    let stream = TcpStream::connect(connect)?;
    let mut writer = stream.try_clone()?;
    let reader = BufReader::new(stream);

    let dbus_session = Connection::session()?;
    let notify_iface = dbus::NotificationsProxyBlocking::new(&dbus_session)?;

    println!("login {user}@{hostname}\r\nconsume\r\n");
    writer.write_all(format!("login {user}@{hostname}\r\nconsume\r\n").as_bytes())?;

    let mut details = None;

    for line in reader.lines() {
        let line = line?;
        println!("{}", line);
        //if line.is_empty() { continue }
        let (_, msg) = protocol::parser::line(&line, false).unwrap();
        let cmd = msg.command.to_uppercase();
        match cmd.as_ref() {
            "NOTIFY_START" => {
                details = Some(NotificationDetails::new());
                if let Some(ref mut details) = details {
                    details.user = msg.arguments.first().cloned();
                    details.id = msg
                        .arguments
                        .get(1)
                        .cloned()
                        .map(|s| s.parse::<usize>().unwrap());
                }
            }
            "TITLE" => {
                if let Some(ref mut details) = details {
                    details.title = msg.trailing;
                }
            }
            "BODY" => {
                if let Some(ref mut details) = details {
                    if let Some(body) = &mut details.body {
                        body.push_str(&msg.trailing.unwrap());
                        body.push('\n');
                    } else {
                        details.body = msg.trailing.map(|mut s| {
                            s.push('\n');
                            s
                        });
                    }
                }
            }
            "NOTIFY_END" => {
                if let Some(details) = details {
                    let _ = display(details, &notify_iface)?;
                }
                details = None;
            }
            _ => {}
        }
    }
    Ok(())
}

fn display(
    notification: NotificationDetails,
    iface: &NotificationsProxyBlocking,
) -> anyhow::Result<()> {
    eprintln!("{notification:?}");
    iface.notify(
        &notification.user.unwrap_or(String::from("notificationd")),
        0,
        "dialog-information",
        &notification.title.unwrap_or(String::from("")),
        &notification.body.unwrap_or(String::from("")),
        &[],
        HashMap::new(),
        0,
    )?;
    Ok(())
}
