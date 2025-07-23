use core::time;
use std::{os::unix::thread, path::Path, process::exit};

use prompted::{input};
mod data_finder;

fn main() {
    let folder_path = input!("Enter the path to your ChatGPT data folder: ");
    // let folder_path = "/home/nevalaonni/Downloads/f6f1d67c721c27dc9844a97f184c77ef92b167ae74d1d74a7c89d04a57e251fd-2025-07-23-13-16-55-476e3ecc48c74823a227fc067f28efa3";

    let path = Path::new(&folder_path);
    
    if !path.exists() {
        println!("Please enter a valid path!");
        exit(-1)
    }

    let feedback: data_finder::Feedback = data_finder::find_feedback(path);
    println!("Positive feedback: {}",feedback.positive_amount);
    println!("Negative feedback: {}",feedback.negative_amount);

    let conversations = data_finder::load_conversations(path);
    
    let analysis: data_finder::Analysis = data_finder::analyze_conversations(conversations);

    println!("Amount of chats: {}", analysis.chat_amount);
    println!("Unfinished messages: {}", analysis.unfinished_messages);
    println!("CHATGPT messages: {}", analysis.messages_from_chatgpt);
    println!("USER messages: {}", analysis.messages_from_user);
    println!("First message: {} - ({})", analysis.oldest_message_time, analysis.oldest_message_id);
}

