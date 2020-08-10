#![recursion_limit = "1048576"]
#![feature(trivial_bounds)]

use console_error_panic_hook;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
use serde::Deserialize;
use std::fmt::Debug;
use std::panic;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys;
use yew::prelude::*;
use yew::services::{reader, reader::FileData, ConsoleService};

#[derive(Debug, Deserialize, Clone, Default)]
struct Record {
    #[serde(rename = "Timestamp", default)]
    timestamp: String,
    #[serde(rename = "Email Address", default)]
    submit_email: String,
    #[serde(rename = "First Name", default)]
    first_name: String,
    #[serde(rename = "Last Name", default)]
    last_name: String,
    #[serde(rename = "UNC Charlotte ID (800#) ", default)]
    candidate_id: String,
    #[serde(rename = "Email Address", default)]
    candidate_email: String,
    #[serde(rename = "Phone Number", default)]
    phone_no: String,
    #[serde(rename = "Student Status", default)]
    student_status: String,
    #[serde(rename = "Gender", default)]
    gender: String,
    #[serde(rename = "Degree Program", default)]
    degree: String,
    #[serde(rename = "Current Advisor", default)]
    advisor: String,
    #[serde(rename = "Date Program Entered", default)]
    date_program_entered: String,
    #[serde(rename = "GPA", default)]
    gpa: String,
    #[serde(rename = "Credit Hours Completed", default)]
    credit_hours: String,
    #[serde(rename = "Currently Working on Campus?", default)]
    currently_working: String,
    #[serde(rename = "Supervisor", default)]
    supervisor: String,
    #[serde(rename = "Department", default)]
    department: String,
    #[serde(rename = "Position", default)]
    position: String,
    #[serde(rename = "Courses Qualified to Grade", default)]
    qualified_for: String,
    #[serde(
        rename = "Other skills or information you would like to provide (e.g.  Dean's List, Chancellor's List, Prior TA experience, etc.)",
        default
    )]
    other: String,
    #[serde(rename = "Upload CV or resume (Optional)", default)]
    resume: String,
    #[serde(default)]
    _index: u32,
    #[serde(default)]
    score: i64,
}

struct Model {
    link: ComponentLink<Self>,
    file_parsed: bool,
    file_reader_service: reader::ReaderService,
    tasks: Vec<reader::ReaderTask>,
    records: Vec<Record>,
    clean_records: Vec<String>,
    sorted_records: Vec<u32>,
}

enum Msg {
    SearchInput(String),
    FileInput,
    FileRead(FileData),
}

impl Model {
    fn file_parsed(&self) -> Html {
        if self.file_parsed {
            html! { <pre> { format!("      ✅ found {} records", self.records.len()) } </pre> }
        } else {
            html! {}
        }
    }

    fn render_records(&self) -> Html {
        html! {
            <pre> { 
                self.sorted_records
                    .iter()
                    .rev()
                    .map(|i| self.clean_records[*i as usize].as_str())
                    .collect::<Vec<&str>>()
                    .join("\n\n")
            } </pre>
        }
    }
}

impl Component for Model {
    type Message = Msg;
    type Properties = ();
    fn create(_: Self::Properties, link: ComponentLink<Self>) -> Self {
        Self {
            link,
            file_parsed: false,
            file_reader_service: reader::ReaderService::new(),
            tasks: vec![],
            records: vec![],
            clean_records: vec![],
            sorted_records: vec![],
        }
    }

    fn update(&mut self, msg: Self::Message) -> ShouldRender {
        match msg {
            Msg::SearchInput(val) => {
                let matcher = SkimMatcherV2::default();

                for (i, r) in self.clean_records.iter().enumerate() {
                    let score = matcher.fuzzy_match(r, &val).unwrap_or(0);
                    self.records[i as usize].score = score;
                }

                let mut sorted_records = self.records.clone();
                sorted_records.sort_unstable_by_key(|r| r.score);

                self.sorted_records = sorted_records.iter().map(|r| r._index).collect();
            }
            Msg::FileInput => {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let file = document
                    .get_element_by_id("fileInput")
                    .expect("Cannot find element with ID 'fileInput'")
                    .dyn_into::<web_sys::HtmlInputElement>()
                    .expect("Element with ID 'fileInput' is not an HtmlInputElement")
                    .files()
                    .expect("No FilesList on found HtmlInputElement")
                    .get(0)
                    .expect("FilesList is empty.");

                let reader_task = self
                    .file_reader_service
                    .read_file(file, self.link.callback(|e| Msg::FileRead(e)))
                    .expect("Can't Read?");

                self.tasks.push(reader_task);
            }
            Msg::FileRead(data) => {
                let text = String::from_utf8(data.content).unwrap();
                
                let text = text.replace("MS in Computer Science", "Masters in Computer Science");
                let text = text.replace("BS in Computer Science", "Bachelors in Computer Science");

                let mut rdr = csv::ReaderBuilder::new().from_reader(text.as_bytes());

                let mut index = 0;
                for record in rdr.deserialize::<Record>() {
                    match record {
                        Ok(r) => {
                            index = index + 1;

                            let qualified_for = r.qualified_for;
                            let qualified_for = qualified_for
                                .split_whitespace()
                                .collect::<Vec<&str>>()
                                .join(" ")
                                .split(",")
                                .collect::<Vec<&str>>()
                                .join("\n");
                            

                            let r = Record {
                                _index: index,
                                qualified_for: qualified_for,
                                ..r
                            };
                            let mut clean = String::new();

                            clean.push_str(format!("{}.\n", r._index).as_str());

                            if r.first_name != "" {
                                clean.push_str(
                                    format!("• First Name:     {}\n", r.first_name).as_str(),
                                );
                            }

                            if r.last_name != "" {
                                clean.push_str(
                                    format!("• Last Name:      {}\n", r.last_name).as_str(),
                                );
                            }

                            if r.candidate_id != "" {
                                clean.push_str(
                                    format!("• Student ID:     {}\n", r.candidate_id).as_str(),
                                );
                            }

                            if r.candidate_email != "" {
                                clean.push_str(
                                    format!("• Email:          {}\n", r.candidate_email).as_str(),
                                );
                            }

                            if r.phone_no != "" {
                                clean.push_str(
                                    format!("• Phone Number:   {}\n", r.phone_no).as_str(),
                                );
                            }

                            if r.student_status != "" {
                                clean.push_str(
                                    format!("• Student Status: {}\n", r.student_status).as_str(),
                                );
                            }

                            if r.degree != "" {
                                clean
                                    .push_str(format!("• Degree Program: {}\n", r.degree).as_str());
                            }

                            if r.date_program_entered != "" {
                                clean.push_str(
                                    format!("• Program Entry:  {}\n", r.date_program_entered)
                                        .as_str(),
                                );
                            }

                            if r.gpa != "" {
                                clean.push_str(format!("• GPA:            {}\n", r.gpa).as_str());
                            }

                            if r.credit_hours != "" {
                                clean.push_str(
                                    format!("• Credit Hours:   {}\n", r.credit_hours).as_str(),
                                );
                            }

                            if r.currently_working != "" {
                                clean.push_str(
                                    format!("• Working?:       {}\n", r.currently_working).as_str(),
                                );
                            }

                            if r.qualified_for != "" {
                                let mut temp = String::new();
                                temp.push_str("• Qualified For:\n");
                                for row in r.qualified_for.split("\n") {
                                    temp.push_str(
                                        format!("                 • {}\n", row.trim()).as_str(),
                                    );
                                }
                                clean.push_str(temp.as_str());
                            }

                            if r.other != "" {
                                clean.push_str(
                                    format!("• Other:          {}\n", r.other.trim()).as_str(),
                                );
                            }

                            self.clean_records.push(clean);
                            self.records.push(r);

                            self.file_parsed = true;
                        }
                        Err(e) => ConsoleService::log(format!("{:?}", e).as_str()),
                    }
                }
                
                self.clean_records.push(String::new());
                self.records.push(Record::default());
                self.sorted_records = self.records.iter().map(|r| r._index).collect();
            }
        }
        true
    }

    fn change(&mut self, _props: Self::Properties) -> ShouldRender {
        // Should only return "true" if new properties are different to
        // previously received properties.
        // This component has no properties so we will always return "false".
        true
    }

    fn view(&self) -> Html {
        html! {
            <div>
                <div class={"padded"}>
                    <pre style={"display: inline"}>{"CSV:  "}</pre>
                    <input
                        type="file"
                        style={"display: inline"}
                        oninput=self.link.callback(|_| Msg::FileInput)
                        multiple=false
                        accept=".csv"
                        id="fileInput"
                    />
                    <br />
                    { self.file_parsed() }
                </div>

                <div class={"padded"}>
                    {"Find: "}
                    <input
                        type="text"
                        oninput=self.link.callback(|e: InputData| Msg::SearchInput(e.value))
                        style={"display: inline"}
                    />

                    { self.render_records() }
                </div>
            </div>
        }
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    panic::set_hook(Box::new(console_error_panic_hook::hook));

    App::<Model>::new().mount_to_body();
}
