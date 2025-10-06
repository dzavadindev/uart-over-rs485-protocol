use serialport::*;

fn main() {
    println!("Started the client. . .");

    // Make sure to adjust for your port if using Win/MacOS. I am testing this in Linux
    const PATH: &str = "/dev/ttyACM0";
    const BAUD: u32 = 9600;

    let mut port = serialport::new(PATH, BAUD).open().expect("Unable to open");

    println!("---------------------");
    println!("Baudrate: {0}", port.baud_rate().unwrap());
    println!("Data Bits: {0}", port.data_bits().unwrap());
    println!("Flow Control: {0}", port.flow_control().unwrap());
    println!("Parity: {0}", port.parity().unwrap());
    println!("Timeout: {:?}", port.timeout());
    println!("---------------------");

    loop {
        println!("1. Change LED blink rate");
        println!("2. Reboot the MCU");
        println!("---------------------");

        let mut raw: String = String::new();
        match std::io::stdin().read_line(&mut raw) {
            Ok(_) => {}
            Err(err) => println!("Error: {0}", err),
        }; // This is blocking, waiting for user input

        let option = match raw.trim().parse::<u8>() {
            Ok(parsed) => parsed,
            Err(_) => {
                println!("{0} is not a valid option", raw.trim());
                continue;
            }
        };

        match option {
            1 => {
                if !handle_blink_rate(&mut port) {
                    continue;
                }
            }
            2 => {
                let tx_buf: [u8; 4] = [0xFF, 0x2, 0x0, 0xFE]; // The 0x2 is the REBOOT command. Data is a 0 byte because it doesn't matter in this case  
                let written = port.write(&tx_buf).unwrap();
                println!("Written {0} bytes", written);
            }
            _ => println!("Invalid option"),
        }
    }
}

fn handle_blink_rate(port: &mut Box<dyn SerialPort>) -> bool {
    let mut new_rate = String::new();

    println!("New blink rate: ");
    match std::io::stdin().read_line(&mut new_rate) {
        Ok(_) => {}
        Err(err) => println!("Error: {0}", err),
    }; // This is blocking, waiting for user input

    let new_rate = match new_rate.trim().parse::<u8>() {
        Ok(parsed) => parsed,
        Err(_) => {
            println!(
                "Failed to parse {0} into a number. Try 0 to 255",
                new_rate.trim()
            );
            return false;
        }
    };

    // Here, we write the start byte, the command byte (BLINK_RATE), data, and an end byte.
    let tx_buf: [u8; 4] = [0xFF, 0x1, new_rate, 0xFE];

    let written = port.write(&tx_buf).unwrap();
    println!("Written {0} bytes", written);
    true
}
