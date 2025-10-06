# UART over RS-485 Protocol

This project is an implementation of a custom communication protocol on top of UART/RS-485

## Time taken

Research: \~3 Hours

Implementation: \~8 Hours

TOTAL: 11 Hours over 3 days.

By the 3 hour mark of the implementation I have completed:

- single byte bi-directional transfers over UART 
- the design completed (without a consideration for RS-485 yet, rewritten to the current version later).

After those 3 hours I havent even gotten to the interesting part with the PC defining the blink rate of an LED, so I just decided to work on it for fun and to wrap my head around the things I got stuck on and try complete more of the assignment.

I have completed the design and the code part, up to task 3. I started to think how I could realise the code for the 4th task, with a separate thread listening to the transfers and printing the notification to the console every time the debounced button is pressed, but I think I took long enough already, and I am not to comfortable with asynchronous Rust (or interrupts in embedded Rust, for that matter. That's why I decided to use `nb` crate after seeing that the HAL I am using also makes use of it).

## Protocol

The assignment describes a simple two-way communication, but the protocol is intended to work over RS-485, which is a multi-drop standard, so the protocol must work with multiple devices on the same bus. 

I don't have an RS-485 transceiver to play around with, so the code provided for both sides assumes 1-1 relationship. *And because of that the code uses a version of the protocol without the addressing spec*

The things that would have been different in code, have I had access to a RS-485 transceiver would be:

- Treat the system as a master->slaves relationship
- Make sure to manage bus collisions with DE/RE pins, by making the master node dictate when its sending packets and when its listening for them
- For slave->master communication, instead of the slave device writing a NOTIFY packets to its TX buffer (because there is none with RS-485), I would have looked into a single master (probably the PC, because its ultimately faster with processing) polling all the connected slave devices on a timer, and executing requests they may have made to the master device.

## Description

For the things it aims to achieve, I reckon it would be most reasonable to think of each transaction as a 'command', where an initiator sends a command and the receiver will process and execute something upon receiving a command.

Then, to make it possible for both pc->mcu and mcu->pc communication, the pc is to be the systems master node. The master node would both send packets to the slave devices and poll every slave device for requests, if they have any. (Much later after writing this I have found out this is exactly how modbus works :thumbsup:)

### Packet

The structure of every MASTER packet will look like this:
```
[START 1byte][ADDRESS 1byte][PAYLOAD 1byte][DATA 1byte][END 1byte]
```

START is a single 0xFF byte to signify the start of the packet.

ADDRESS is the byte long address of the device the packet is meant for. Every device has its own address. Master doesn't have an address dedicated to it, as every request a slave device can send is to the master node. Probably the best way for smaller networks is to load the address to the devices EEPROM.

PAYLOAD is the command byte. Every command has a byte value that represents it.

DATA is mainly for BLINK_RATE command, so its possible to pass the new desired blink rate. Doesn't play any role for other two commands, and is practically ignored.

END is a single 0xFE byte to signify the end of a packet.

The structure of a SLAVE packet/request will look like this:
```
[START 1byte][PAYLOAD 1byte][END 1byte]
```

Same thing for START and END, and a PAYLOAD byte in between only really can have 1 command variation in this version - 0x3.

The slave packets lack the ADDRESS byte, because there is no use-case for slave->slave communication I can see. So any slave request is just handled by master. And even then, polling would probably be weird here, would need to rethink how the bus is being controlled to accomodate slave->slave communication.

And the slave packets lack the DATA byte, because again, the only meaningful command they can send is 0x3/NOTIFY, which doesn't have a DATA due to the trigger being a simple button press. With a more complex input interface on the boards side, there might have been a reason to allow for a DATA byte for the NOTIFY message encoded with ASCII, for example, because the capacity is 1 byte.

### Messages

The 3 messages are BLINK_RATE, REBOOT, and NOTIFY.

- BLINK_RATE | PC -> MCU
COMMAND BYTE: 0x1
The BLINK_RATE message contains a byte long payload in the DATA part of the packet. That DATA defines the new blink frequency of the LED. The DATA is an byte, thus the blink values to set are from 0 (no blinking) to 255 (~2 second on and off cycle)

- REBOOT | PC -> MCU
COMMAND BYTE: 0x2
The REBOOT message. Slef-explanatory - reboots the MCU.

- NOTIFY | MCU -> PC
COMMAND BYTE: 0x3
The NOTIFY message is an indication of a button press. It does not carry a payload, as there is no way to enter a message payload on the board.

## Remarks

If you would have time for it, I would love to see some feedback on the design and the code. I can only assume that I have missed multiple (non)obvious caveats or drawbacks to my approaches.


## Research (Reference for myself)

I was not familiar with RS-485 before making the project, thus first thing was to get familiar with what it is.

Before getting into the project, with knowledge from my course material, I though of UART as a full-fletched protocol, and looking into RS-485 gave some interesting insight.

"Plain UART" is not really a full-fletched protocol, and actually a "framing layer" of communication. Overall, there are 3 communication layers: 
- Physical

Self-explanatory, the "physical" part of communication, e.g. the wiring and voltages

- Framing

Defines how to read a byte and how to address transmissions.

- Application (Protocol)

The actual meaning behind the transmission messages. What one usually would call a "protocol"

UART is usually put in the same list as I2C and SPI, but where those two define both the physical and the framing layers, UART only covers the framing. Thus there is flexibility in what to pick as your physical layer to match system specs, like for example (what I reckon most people mean when saying "UART") UART-over-TTL-UART, which is standard, low-range, one-to-one (RX/TX) communication, usually used for connecting modules to the MCU.

RS-485 is a way transmit digital signals as differential voltages over a twisted pair.Because of its design it supports larger distances and multi-drop communication and its also more noise resistant.
