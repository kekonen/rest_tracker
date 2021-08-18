use chrono::{Local, Duration};
use chrono::prelude::*;
use std::thread;
use std::cell::RefCell;
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq, Eq)]
enum TaskState {
    Initiated,
    InitialWait,
    HalfedExtension,
    QuarteredExtension,
    Failed,
}

#[derive(Debug)]
struct TimeRange {
    from: DateTime<Local>,
    to: DateTime<Local>,
}

impl TimeRange {
    fn new(from: DateTime<Local>, to: DateTime<Local>) -> Self {
        Self {
            from, to
        }
    }
}

#[derive(Debug)]
struct Task {
    state: RefCell<TaskState>,
    gap: i64,
    phase: RefCell<Option<TimeRange>>,
}

impl Task {
    fn new(gap: i64) -> Self {
        Self {
            state: RefCell::new(TaskState::Initiated),
            gap,
            phase: RefCell::new(None),
        }
    }

    fn next(&self, divider: Option<i64>, state: TaskState) {
        match state {
            TaskState::Failed => {
                *self.phase.borrow_mut() = None;
            },
            _ => {
                let now = Local::now();
                let phase = TimeRange {
                    from: now,
                    to: now + Duration::milliseconds(self.gap/divider.expect("None can only be when the state is Failed"))
                };

                *self.phase.borrow_mut() = Some(phase);
            }
        }
        *self.state.borrow_mut() = state;
    }

    fn is_elapsed(&self) -> bool {
        if let Some(ref phase) = *self.phase.borrow() {
            Local::now() > phase.to 
        } else {
            true
        }
    }

    fn till(&self) -> Option<DateTime<Local>> {
        if let Some(ref phase) = *self.phase.borrow() {
            Some(phase.to) 
        } else {
            None
        }
    }

    // Do something when elapsed
    fn move_next_state(&self) {
        let state = self.state.borrow().clone();
        match state {
            TaskState::Initiated => {
                self.next(Some(1), TaskState::InitialWait);
            },
            TaskState::InitialWait => {
                self.next(Some(2), TaskState::HalfedExtension);
            },
            TaskState::HalfedExtension => {
                self.next(Some(4), TaskState::QuarteredExtension);
            },
            TaskState::QuarteredExtension => {
                self.next(None, TaskState::Failed);
            },
            TaskState::Failed => {
            }
        }
    }

    fn peek(&self) -> TaskState {
        if self.is_elapsed() {
            self.move_next_state()
        }
        self.state.borrow().clone()
    }
}

fn main() {
    let got_line: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));
    let got_line_mx = got_line.clone();
    let done: Arc<Mutex<bool>> = Arc::new(Mutex::new(false));


    thread::spawn(move || {
        let stdin = io::stdin();
        let mut iterator = stdin.lock().lines();

        loop {
            let done = *done.lock().unwrap();
            if !done {
                let line1 = iterator.next().unwrap().unwrap();
                *got_line_mx.lock().unwrap() = true;
            } else {
                break
            }
            
        }
    });

    for _ in 1..20 {

        let task = Task::new(1000_i64 * 25 * 60);
        let mut previous_state = task.peek();
        println!("Started a new task! till: {}", task.till().unwrap().format("%H:%M:%S").to_string());

        loop {
            thread::sleep(std::time::Duration::from_millis(500));

            let line = *got_line.lock().unwrap();
            if line {
                *got_line.lock().unwrap() = false;
                println!("Done!!!!");
                break
            }
    
            
            let state = task.peek();
            if state != previous_state {
                match state {
                    TaskState::InitialWait => {
                        println!("Time started! till: {}", task.till().unwrap().format("%H:%M:%S").to_string())
                    },
                    TaskState::HalfedExtension => {
                        println!("Time elapsed! extending half! till: {}", task.till().unwrap().format("%H:%M:%S").to_string())
                    },
                    TaskState::QuarteredExtension => {
                        println!("Time elapsed! extending quarter! till: {}", task.till().unwrap().format("%H:%M:%S").to_string())
                    },
                    TaskState::Failed => {
                        println!("Failed!");
                        break
                    },
                    _ => {},
                }
            }
            
            previous_state = state;
        }
    }
    

}
