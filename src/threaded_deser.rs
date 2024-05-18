use std::{
    collections::VecDeque,
    fs::File,
    io::{BufReader, Cursor},
    num::NonZeroUsize,
    sync::{Arc, Condvar, Mutex},
    thread,
};

use quick_xml::events::{BytesEnd, Event};

use crate::{generation_error, Cores, ManycoreError, ManycoreSystem};

const BUFFER_CAPACITY: usize = 4096; // 4kb
const CORES_PER_THREAD: usize = 100;

impl ManycoreSystem {
    pub(crate) fn threaded_deserialise(path: &str) -> Result<(), ManycoreError> {
        let file = File::open(path).map_err(|e| generation_error(e.to_string()))?;
        let reader = BufReader::with_capacity(BUFFER_CAPACITY, file);

        let mut xml_reader = quick_xml::Reader::from_reader(reader);
        let mut xml_writer = quick_xml::Writer::new(Cursor::new(Vec::new()));
        let mut without_cores = Vec::with_capacity(BUFFER_CAPACITY);
        let mut cores: Vec<u8> = Vec::new();

        // let mut q = VecDeque::new();
        let max_threads =
            usize::from(thread::available_parallelism().unwrap_or(NonZeroUsize::new(1).unwrap()));
        let threading_semaphore = Arc::new((Mutex::new(0usize), Condvar::new()));

        let _ = thread::scope(|s| -> Result<(), ManycoreError> {
            loop {
                let evt = xml_reader
                    .read_event_into(&mut without_cores)
                    .map_err(|e| generation_error(e.to_string()))?;
                
                match evt {
                    Event::Start(ref e) => {
                        if e.name().as_ref() == b"Cores" {
                            let mut count = 0;
                            loop {
                                match xml_reader
                                    .read_event_into(&mut cores)
                                    .map_err(|e| generation_error(e.to_string()))?
                                {
                                    Event::End(ref e) => match e.name().as_ref() {
                                        b"Core" => {
                                            count += 1;

                                            if count == CORES_PER_THREAD {
                                                cores.extend(b"</Cores>".into_iter());
                                                let moved_cores = cores.clone();
                                                let moved_threading_semaphore =
                                                    threading_semaphore.clone();
                                                s.spawn(move || {
                                                    let (sem_tokens_mutex, sem_control) =
                                                        &*moved_threading_semaphore;
                                                    // Unwrapping because we don't panic
                                                    let mut sem_tokens =
                                                        sem_tokens_mutex.lock().unwrap();
                                                    while *sem_tokens == max_threads {
                                                        // Again, unwrapping because we are not panicking
                                                        sem_tokens =
                                                            sem_control.wait(sem_tokens).unwrap();
                                                    }

                                                    // let tmp: Cores = quick_xml::de::from_reader(
                                                    //     &moved_cores[..],
                                                    // )
                                                    // .unwrap();
                                                    // println!("{tmp:?}");
                                                });

                                                cores.clear();
                                                cores.extend(b"<Cores>".into_iter());
                                                count = 0;
                                            }
                                        }
                                        _ => {
                                            println!("Done with reading cores");
                                            xml_writer
                                                .write_event(Event::End(BytesEnd::new("Cores")))
                                                .unwrap();
                                            break;
                                        }
                                    },
                                    _ => {}
                                }
                            }
                        } else {
                            xml_writer.write_event(evt).unwrap();
                        }
                    }
                    Event::Eof => {
                        break;
                    }
                    _ => {}
                }

                without_cores.clear();
            }

            Ok(())
        });

        println!(
            "{}",
            std::str::from_utf8(xml_writer.into_inner().into_inner().as_slice()).unwrap()
        );

        Ok(())
    }
}
