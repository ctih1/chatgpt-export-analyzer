use std::{collections::HashMap, path::Path, process::exit, time::{Duration, SystemTime}};


use chrono::{DateTime, Datelike, Local, NaiveDate, NaiveTime};
use crossterm::event::{self, Event, KeyCode};
use prompted::{input};
use ratatui::{layout::{ Constraint, Layout}, style::{Color, Style, Stylize}, text::{Line,  Text}, widgets::{ BarChart, Block, List,  Tabs}, Frame};
use Constraint::{Fill, Length, Min};

use crate::data_finder::{Analysis, Feedback};

mod data_finder;


fn main() {
    let folder_path = input!("Enter the path to your ChatGPT data folder: ");

    let path = Path::new(&folder_path);
    
    if !path.exists() {
        println!("Please enter a valid path!");
        exit(-1)
    }

    let feedback: data_finder::Feedback = data_finder::find_feedback(path);

    let mut start = SystemTime::now();
    println!("Loading JSON");
    let conversations = data_finder::load_conversations(path);
    println!("Loaded JSON in {}ms", start.elapsed().expect("Invalid time decoding").as_millis().to_string());

    start = SystemTime::now();

    println!("Analyzing JSON...");
    let analysis: data_finder::Analysis = data_finder::analyze_conversations(conversations);
    println!("Analyzed in {}ms", start.elapsed().expect("Invalid time decoding").as_millis().to_string());
    println!("Raw timestamps sample: {:?}", &analysis.messages_sent.iter().take(5).collect::<Vec<_>>());

    let mut selected_tab = 0;
    let tabs = vec!["Basic data", "Usage", "Resources"];

    let mut terminal = ratatui::init();
    let mut last_button_click = SystemTime::now();

    loop {
        terminal.draw(|f| draw(f, &analysis, &feedback, selected_tab, &tabs)).expect("failed to draw frame");
        if let Event::Key(key) = event::read().expect("failed to read event") {
            match key.code {
                KeyCode::Char('q') => break,
                KeyCode::Right => {
                    if SystemTime::now().duration_since(last_button_click).expect("Invalid button click time!") < Duration::from_millis(100) {
                        continue;
                    }
                    last_button_click = SystemTime::now();
                    if selected_tab == tabs.len() - 1 {
                        selected_tab = 0;
                    } else {
                        selected_tab += 1
                    }
                }
                KeyCode::Left => {
                    if SystemTime::now().duration_since(last_button_click).expect("Invalid button click time!") < Duration::from_millis(100) {
                        continue;
                    }
                    last_button_click = SystemTime::now();
                    if selected_tab == 0 {
                        selected_tab = tabs.len() -1
                    } else {
                        selected_tab -= 1
                    }
                }

                _ => {}
            }
        }
    }
    ratatui::restore();
}

fn hashmap_to_ordered_vec(map: &HashMap<String, i32>) -> Vec<String> {
    let mut sorted: Vec<_> = map.clone().into_iter().collect();
    sorted.sort_by_key(|&(_, uses)| uses);

    let mut results: Vec<String> = sorted.into_iter().map(|(model, uses)| format!("{}: {}", model, uses)).collect();
    results.reverse();

    return results
}

fn draw(frame: &mut Frame, analysis: &Analysis, feedback: &Feedback, selected_tab: usize, tabs: &Vec<&'static str>) {
    let vertical = Layout::vertical([Length(3), Min(0), Length(1)]);
    let [title_area, mid_area, status_area] = vertical.areas(frame.area());

    let main_split = Layout::vertical([Fill(1), Fill(1)]);
    let [top_area, bottom_area] = main_split.areas(mid_area);

    let horizontal_top = Layout::horizontal([Fill(1), Fill(1)]);
    let [left_area, right_area] = horizontal_top.areas(top_area);

    let horizontal_bottom = Layout::horizontal([Fill(1), Fill(1)]);
    let [left_bottom, right_bottom] = horizontal_bottom.areas(bottom_area);

    let quarter_bottom = Layout::horizontal([Fill(1), Fill(1), Fill(1), Fill(1)]);
    let [bottom_quarter_one, bottom_quarter_two, bottom_quarter_three, bottom_quarter_four] = quarter_bottom.areas(bottom_area);

    let bottoms = vec![bottom_quarter_one, bottom_quarter_two, bottom_quarter_three, bottom_quarter_four];


    let tabs = Tabs::new(tabs.iter().cloned().map(Line::from).collect::<Vec<_>>())
        .select(selected_tab)
        .block(Block::bordered().title("Tabs"))
        .highlight_style(Style::new().fg(Color::Green).bold());



    frame.render_widget(tabs, title_area);
    frame.render_widget(Line::from("Q to quit, <- and -> to change tabs"), status_area);


    match selected_tab {
        0 => {
            let model_list = List::new(hashmap_to_ordered_vec(&analysis.models_used))
                .block(Block::bordered().title("Models prompted"))
                .style(Style::default().fg(Color::Magenta));

            let content_list = List::new(hashmap_to_ordered_vec(&analysis.content_types))
                .block(Block::bordered().title("Content types used"))
                .style(Style::default().fg(Color::LightBlue));

            let chat_info = List::new(vec![
                format!("Total number of chats: {}", analysis.chat_amount),
                format!("Total number of messages you've sent: {}", analysis.messages_from_user),
                format!("Number of disrupted chats: {}", analysis.unfinished_messages),
                format!("Chats from ChatGPT (not accurate): {}", analysis.messages_from_chatgpt),
                format!("Your first message: {}", DateTime::from_timestamp(analysis.oldest_message_time.trunc() as i64 ,0).expect("Invalid oldest date!").format("%H:%M:%S %d/%m/%Y")),
                format!("First chat link (may not work): https://chatgpt.com/c/{}", analysis.oldest_message_id),
                format!("Voices used: {}", analysis.voices_used.join(",")),
                format!("Feedback positivity: {}% (positive: {}, negative: {})", feedback.positive_amount as f64 / (feedback.positive_amount as f64 + feedback.negative_amount as f64) * 100.0, feedback.positive_amount, feedback.negative_amount)
            ])
                .block(Block::bordered().title("Basic information"))
                .style(Style::default().fg(Color::Green));


            let authors = List::new(hashmap_to_ordered_vec(&analysis.authors))
                .block(Block::bordered().title("Prompt response sources"))
                .style(Style::default().fg(Color::DarkGray));



                
            frame.render_widget(model_list, left_area);
            frame.render_widget(content_list, right_area);
            frame.render_widget(chat_info, left_bottom);
            frame.render_widget(authors, right_bottom);

        }

        1 => {
            let mut date_counts: HashMap<(i32, u32), u64> = HashMap::new();
            
            for timestamp in &analysis.messages_sent {
                if let Some(datetime) = DateTime::from_timestamp(timestamp.trunc() as i64, 0) {
                    let naive = datetime.naive_utc();
                    *date_counts.entry((naive.year(), naive.month())).or_insert(0) += 1;
                }
            }

            let mut time_data_points: Vec<(f64, f64)> = date_counts
                .iter().map(|((year, month), count)| {
                        (NaiveDate::from_ymd_opt(*year, *month, 1).unwrap().and_time(NaiveTime::default()).and_local_timezone(Local).earliest().unwrap().num_days_from_ce() as f64, *count as f64)
                    }).collect();

            time_data_points.sort_by(|a, b| a.0.total_cmp(&b.0));


            let mut string_labels = Vec::new();
            let mut bar_data = Vec::new();

            for (x, _y) in &time_data_points {
                let days = *x as i32;
                let date = chrono::NaiveDate::from_num_days_from_ce_opt(days).unwrap();
                let label = date.format("%m-%y").to_string();
                string_labels.push(label); // Own the String
            }
            for (label, y) in string_labels.iter().zip(time_data_points.iter().map(|(_, y)| *y as u64)) {
                bar_data.push((label.as_str(), y));
            }

            let chart = BarChart::default()
                .block(Block::bordered().title("Messages per month (month-year)"))
                .bar_width(5)
                .data(&bar_data)
                .style(Style::default().fg(Color::Green));

            frame.render_widget(chart, mid_area);

        }

        2 => {
            let visited_websites = List::new(hashmap_to_ordered_vec(&analysis.searched_websites))
                .block(Block::bordered().title("Visited websites"))
                .style(Style::default().fg(Color::LightMagenta));

            let visited_pages = List::new(hashmap_to_ordered_vec(&analysis.website_paths))
                .block(Block::bordered().title("Visited paths"))
                .style(Style::default().fg(Color::Cyan));


            let word_map = hashmap_to_ordered_vec(&analysis.words);
            
            let mut word_list_vec: Vec<List> = vec![];
            let word_page_amount = 12;
            for page in 0..4 {
                word_list_vec.push(
                    List::new(word_map[page*word_page_amount..page*word_page_amount + word_page_amount].to_vec())
                        .block(Block::bordered().title(format!("Words in prompts ({})", page+1)))
                        .style(Style::default().fg(Color::Yellow))
                );
                
            }

            frame.render_widget(visited_websites, left_area);
            frame.render_widget(visited_pages, right_area);

            for (index, word) in word_list_vec.iter().enumerate() {
                frame.render_widget(word, bottoms[index]);
            }
        }
        _ => {
            frame.render_widget(Text::from("Invalid tab!").red(), top_area);
        }
    }

    

}