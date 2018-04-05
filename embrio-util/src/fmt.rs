#[macro_export]
macro_rules! await_write {
    ($dst:expr, $buffer:expr, $($arg:tt)*) => ({
        let mut buffer = $buffer;
        let position = {
            let mut cursor = $crate::embrio::io::Cursor::new(&mut buffer[..]);
            write!(cursor, $($arg)*)?;
            cursor.position()
        };
        $crate::futures::prelude::await!(
            $crate::io::write_all(
                Pin::borrow(&mut $dst),
                &buffer[..position]))
    })
}

#[macro_export]
macro_rules! await_writeln {
    ($dst:expr, $buffer:expr, $($arg:tt)*) => ({
        let mut buffer = $buffer;
        let position = {
            let mut cursor = $crate::embrio::io::Cursor::new(&mut buffer[..]);
            writeln!(cursor, $($arg)*)?;
            cursor.position()
        };
        $crate::futures::prelude::await!(
            $crate::io::write_all(
                Pin::borrow(&mut $dst),
                &buffer[..position]))
    })
}
