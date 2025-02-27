use std::{
    fs::File,
    io::{self, BufRead, BufReader, Lines},
};

pub fn read_file_lines(
    filename: &str,
) -> Result<ExtractErrorIterator<String, io::Error, Lines<BufReader<File>>>, io::Error> {
    let file = std::fs::File::open(filename)?;
    let bufreader = std::io::BufReader::new(file);
    Ok(ExtractErrorIterator::new(bufreader.lines()))
}

#[allow(clippy::type_complexity)]
pub fn read_lines(
    writer: &mut dyn io::Read,
) -> Result<ExtractErrorIterator<String, io::Error, Lines<BufReader<&mut dyn io::Read>>>, io::Error>
{
    let bufreader = std::io::BufReader::new(writer);
    Ok(ExtractErrorIterator::new(bufreader.lines()))
}

pub fn extract_error<S, E, I: Iterator<Item = Result<S, E>>>(
    iter: I,
) -> ExtractErrorIterator<S, E, I> {
    ExtractErrorIterator::new(iter)
}

#[derive(Debug)]
pub struct ExtractErrorIterator<S, E, I: Iterator<Item = Result<S, E>>> {
    iter: I,
    err: Option<E>,
}

impl<S, E, I: Iterator<Item = Result<S, E>>> Iterator for ExtractErrorIterator<S, E, I> {
    type Item = S;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.iter.next();
        match item {
            Some(Ok(item)) => Some(item),
            Some(Err(e)) => {
                self.err = Some(e);
                None
            }
            None => None,
        }
    }
}

impl<S, E, I: Iterator<Item = Result<S, E>>> ExtractErrorIterator<S, E, I> {
    pub fn new(iter: I) -> Self {
        Self { iter, err: None }
    }

    pub fn error(self) -> Result<(), E> {
        match self.err {
            Some(err) => Err(err),
            None => Ok(()),
        }
    }
}
