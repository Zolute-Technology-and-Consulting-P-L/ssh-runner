use ssh2::Session;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Check and parse command-line arguments.
    // Usage: <program> <ip/hostname> <port> <username> <key file> <input file>
    let args: Vec<String> = env::args().collect();
    if args.len() != 6 {
        eprintln!(
            "Usage: {} <ip/hostname> <port> <username> <key file> <input file>",
            args[0]
        );
        std::process::exit(1);
    }

    let hostname = &args[1];
    let port = &args[2];
    let username = &args[3];
    let keyfile = &args[4];
    let inputfile = &args[5];

    // Build the address using hostname and port.
    let address = format!("{}:{}", hostname, port);

    // Connect to the server using TcpStream.
    let tcp = TcpStream::connect(address)?;
    let mut sess = Session::new()?;
    sess.set_tcp_stream(tcp);
    sess.handshake()?;

    // Authenticate using the public key.
    sess.userauth_pubkey_file(username, None, Path::new(keyfile), None)?;

    if !sess.authenticated() {
        eprintln!("Authentication failed");
        std::process::exit(1);
    }

    // Open the input file that contains the commands.
    let file = File::open(inputfile)?;
    let reader = BufReader::new(file);

    // Execute each non-empty line as a command on the remote server.
    for line in reader.lines() {
        let command = line?;
        if command.trim().is_empty() {
            continue;
        }
        println!("Executing command: {}", command);

        let mut channel = sess.channel_session()?;
        channel.exec(&command)?;

        // Read the output from the command.
        let mut output = String::new();
        channel.read_to_string(&mut output)?;
        println!("Output:\n{}", output);

        channel.wait_close()?;
        println!("Command exited with status: {}", channel.exit_status()?);
    }

    Ok(())
}
