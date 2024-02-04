use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, SystemTime},
};

use lazy_static::lazy_static;
use rdev::{listen, Event, EventType, Key};
use std::sync::Mutex;

// Define a lazy static variable for a channel to send and receive events.
lazy_static! {
    static ref INPUT_EVENT_CHANNEL: (Mutex<Sender<Event>>, Mutex<Receiver<Event>>) = {
        let (sender, receiver) = mpsc::channel();
        (Mutex::new(sender), Mutex::new(receiver))
    };
}

#[derive(Debug, Clone)]
struct Action {
    key: Key,
    duration: Duration,
    start_time: SystemTime,
}

impl Action {
    fn new(key_press_event: &Event, key_release_event: &Event) -> Self {
        let start_time = key_press_event.time;
        let key = match key_press_event.event_type {
            EventType::KeyPress(key) => key,
            _ => unimplemented!(),
        };
        let duration = key_release_event.time.duration_since(start_time).unwrap();
        Self {
            key,
            duration,
            start_time,
        }
    }
}

// Callback function to be called when an input event is captured.
fn input_event_callback(event: Event) {
    // Send the captured event to the channel.
    INPUT_EVENT_CHANNEL
        .0
        .lock()
        .expect("Failed to lock Event_Channel")
        .send(event)
        .expect("Receiver was stopped");
}

fn main() {
    // Spawn a new thread to listen for input events using the callback function.
    thread::spawn(|| {
        if let Err(error) = listen(input_event_callback) {
            println!("Error: {:?}", error)
        }
    });

    // Lock the receiver from the event channel.
    let receiver = INPUT_EVENT_CHANNEL.1.lock().unwrap();

    let mut buffer: Vec<Event> = vec![];
    // Initialize variables to keep track of previous events.
    let mut previous_event: Option<Event> = None;

    let mut parallel_actions: Vec<Action> = vec![];
    let mut user_action_list: Vec<Vec<Action>> = vec![];

    // Iterate over received events from the channel.
    for current_event in receiver.iter() {
        if let Some(prev_event) = previous_event.clone() {
            // Compare the current event with the previous one.
            if prev_event.event_type == current_event.event_type {
                println!("Same");
            } else {
                println!("Different");

                // Process the event based on its type.
                match current_event.event_type {
                    EventType::MouseMove { .. } => { /* Ignore mouse move events */ }
                    EventType::KeyPress(key) => {
                           // Check if the buffer already contains an event with the same EventType.
                           if !buffer.iter().any(|event| event.event_type == EventType::KeyPress(key)) {
                            // If not, push the current_event into the buffer.
                            buffer.push(current_event.clone());
                            }
                        }
                    EventType::Wheel { .. } => { /* Handle wheel events if needed */ }
                    EventType::KeyRelease(search_key) => {
                        if let Some(index) = buffer.iter().position(|event| {
                            matches!(
                                event.event_type,
                                EventType::KeyPress(key) | EventType::KeyRelease(key) if key == search_key
                            )
                        }) {
                            parallel_actions.push(Action::new(&buffer[index], &current_event));
                            buffer.remove(index);
                            println!("{:?} {}",buffer,buffer.len());
                            if buffer.is_empty() {
                                println!("\n {:?}", parallel_actions);
                                user_action_list.push(parallel_actions.clone());
                                parallel_actions.clear();
                            }
                        }
                    }
                    EventType::ButtonPress(_) | EventType::ButtonRelease(_) => {
                        /* Handle button press or release events if needed */
                    }
                };
            }
        } else {
            println!("Previous event is none");
        }

        // Update the previous event for the next iteration.
        previous_event = Some(current_event.clone());
    }

    // Additional code or actions can be added as needed.
    // Note: The loop is currently infinite, and the program needs an explicit exit condition.
}
