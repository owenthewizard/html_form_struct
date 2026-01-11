use html_form_struct::form_struct;

use fake::{
    Fake,
    faker::{
        internet::en::{Password, SafeEmail, Username},
        number::en::NumberWithFormat,
    },
};

#[form_struct("examples/readme.html", "form#register")]
#[derive(Debug)]
pub struct Registration;

fn main() {
    let username: String = Username().fake();
    let guess = if username.len() % 2 == 0 {
        Some(
            NumberWithFormat("#".repeat(username.len() / 2).as_str())
                .fake::<String>()
                .parse()
                .unwrap(),
        )
    } else {
        None
    };
    let password: String = Password(6..13usize).fake();
    let email = if password.len() % 2 == 0 {
        Some(SafeEmail().fake())
    } else {
        None
    };

    let registration = Registration {
        email,
        guess,
        password,
        username,
    };

    println!("{registration:#?}");
}
