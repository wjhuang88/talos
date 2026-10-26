//! Lossless Windows CRT argument encoding, shared by native launch paths.

const MAX_COMMAND_UNITS: usize = 32_767;

pub(super) fn encode(arguments: &[Vec<u16>]) -> Result<Vec<u16>, String> {
    if arguments.is_empty() {
        return Err("Windows command requires a program argument".into());
    }
    let mut command = Vec::new();
    for (index, argument) in arguments.iter().enumerate() {
        if argument.contains(&0) {
            return Err("Windows command argument contains NUL".into());
        }
        // Check before allocating the escaped representation. Quoting can at
        // most double the input, but no accepted input may exceed this limit.
        if argument.len() >= MAX_COMMAND_UNITS {
            return Err("Windows command line exceeds the platform limit".into());
        }
        if index != 0 {
            command.push(b' ' as u16);
        }
        append_argument(&mut command, argument);
        if command.len() >= MAX_COMMAND_UNITS {
            return Err("Windows command line exceeds the platform limit".into());
        }
    }
    command.push(0);
    Ok(command)
}

fn append_argument(output: &mut Vec<u16>, argument: &[u16]) {
    const QUOTE: u16 = b'"' as u16;
    const SLASH: u16 = b'\\' as u16;
    if !argument.is_empty() && !argument.iter().any(|unit| matches!(*unit, 9 | 32 | QUOTE)) {
        output.extend_from_slice(argument);
        return;
    }
    output.push(QUOTE);
    let mut slashes = 0;
    for &unit in argument {
        if unit == SLASH {
            slashes += 1;
            continue;
        }
        output.extend(std::iter::repeat_n(
            SLASH,
            slashes * if unit == QUOTE { 2 } else { 1 } + usize::from(unit == QUOTE),
        ));
        output.push(unit);
        slashes = 0;
    }
    output.extend(std::iter::repeat_n(SLASH, slashes * 2));
    output.push(QUOTE);
}

#[cfg(test)]
#[path = "windows_command_line_tests.rs"]
mod tests;
