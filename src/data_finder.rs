use std::{collections::HashSet, fs::File, hash::Hash, io::Read, path::Path, time::{SystemTime, UNIX_EPOCH}};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq)]
enum Rating {
    thumbs_down,
    thumbs_up
}

#[derive(Serialize, Deserialize)]
struct FeedBackJson {
    id: String,
    conversation_id: String,
    user_id: String,
    rating: Rating,
    create_time: String,
    workspace_id: Option<String>,
    content: String,
    evaluation_name: Option<String>,
    evaluation_treatment: Option<String>,
    update_time: String
}


#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum AsyncStatus {
    Int(i32),
    Str(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationRoot {
    pub title: String,
    pub create_time: f64,
    pub update_time: f64,
    pub mapping: HashMap<String, MessageNode>,
    pub moderation_results: Vec<serde_json::Value>,
    pub current_node: String,
    pub plugin_ids: Option<serde_json::Value>,
    pub conversation_id: String,
    pub conversation_template_id: Option<String>,
    pub gizmo_id: Option<String>,
    pub gizmo_type: Option<String>,
    pub is_archived: bool,
    pub is_starred: Option<bool>,
    pub safe_urls: Vec<String>,
    pub blocked_urls: Vec<String>,
    pub default_model_slug: Option<String>,
    pub conversation_origin: Option<String>,
    pub voice: Option<String>,
    pub async_status: Option<AsyncStatus>,
    pub disabled_tool_ids: Vec<String>,
    pub is_do_not_remember: Option<bool>,
    pub memory_scope: String,
    pub sugar_item_id: Option<String>,
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageNode {
    pub id: String,
    pub message: Option<Message>,
    pub parent: Option<String>,
    pub children: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub author: Author,
    pub create_time: Option<f64>,
    pub update_time: Option<f64>,
    pub content: MessageContent,
    pub status: String,
    pub end_turn: Option<bool>,
    pub weight: f64,
    pub metadata: MessageMetadata,
    pub recipient: String,
    pub channel: Option<String>,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Author {
    pub role: String,
    pub name: Option<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}



#[derive(Debug, Serialize, Deserialize)]
pub struct MessageContent {
    pub content_type: String,
    pub language: Option<String>,
    pub response_format_name: Option<String>,
    pub thoughts: Option<Vec<HashMap<String, String>>>,
    pub source_analysis_msg_id: Option<String>,

    #[serde(flatten)]
    pub inner: ContentInner,
}


#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ContentInner {
    TextObject { parts: Vec<String> },
    TextField { text: String },
    None {},
    UserEditable {
        user_profile: String,
        user_instructions: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageMetadata {
    #[serde(default)]
    pub is_visually_hidden_from_conversation: Option<bool>,

    #[serde(default)]
    pub user_context_message_data: Option<UserContextData>,

    #[serde(flatten)]
    pub other: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserContextData {
    pub about_user_message: String,
    pub about_model_message: Option<String>,
}



pub struct Feedback {
    pub positive_amount: i32,
    pub negative_amount: i32
}


pub struct Analysis {
    pub chat_amount: i32,
    pub messages_from_chatgpt: i32,
    pub messages_from_user: i32,
    pub unfinished_messages: i32,
    pub models_used: HashMap<String, i32>,
    pub messages_sent: Vec<f64>,
    pub content_types: HashMap<String, i32>,
    pub oldest_message_time: f64,
    pub oldest_message_id: String
}

pub fn find_feedback(path: &Path) -> Feedback {
    let mut file = File::open(path.join("message_feedback.json")).expect("Failed to open message_feedback!");
    let mut content = String::new();

    file.read_to_string(&mut content).expect("Failed to read message_feedback.json content!");
    let feedbacks: Vec<FeedBackJson> = serde_json::from_str(&content).expect("Invalid message_feedback.json formatting!");


    return Feedback { 
        positive_amount: feedbacks.iter().filter(|f| f.rating == Rating::thumbs_up).count() as i32,
        negative_amount: feedbacks.iter().filter(|f| f.rating == Rating::thumbs_down).count() as i32,
    }
}

pub fn load_conversations(path: &Path) -> Vec<ConversationRoot> {
    let mut file = File::open(path.join("conversations.json")).expect("Failed to open conversations!");
    let mut content = String::new();

    file.read_to_string(&mut content).expect("Failed to read conversations.json content!");
    let conversations: Vec<ConversationRoot> = serde_json::from_str(&content).expect("Invalid conversations.json formatting!");
    
    return conversations
}


pub fn analyze_conversations(conversations: Vec<ConversationRoot>) -> Analysis {
    let mut analysis: Analysis = Analysis { 
        chat_amount: conversations.len() as i32,
        messages_from_chatgpt: 0,
        messages_from_user: 0,
        unfinished_messages: 0,
        models_used: HashMap::new(),
        messages_sent: vec![],
        oldest_message_time: SystemTime::now().duration_since(UNIX_EPOCH).expect("Invalid time stamp").as_millis() as f64,
        oldest_message_id: String::new(),
        content_types: HashMap::new()
     };

    for conversation in conversations {
        for (key, value) in conversation.mapping.into_iter() {
            if key == "client-created-root" {
                continue;
            }
            if let Some(message) = value.message {
                if message.author.role == "system" {
                    analysis.messages_from_chatgpt += 1;
                } else {
                    analysis.messages_from_user += 1;
                }

                if message.status != "finished_successfully" {
                    analysis.unfinished_messages += 1;
                }

                *analysis.content_types.entry(message.content.content_type).or_insert(0) += 1;

                if let Some(slug) = message.metadata.other.get("model_slug"){
                    *analysis.models_used.entry(slug.to_string()).or_insert(0) += 1
                }
                
                if let Some(time) = message.create_time {
                    if time < analysis.oldest_message_time {
                        analysis.oldest_message_time = time;
                        analysis.oldest_message_id = message.id
                    }
                    analysis.messages_sent.push(time);
                }
            }
        }
        analysis.chat_amount += 1;
    }

    return analysis
}