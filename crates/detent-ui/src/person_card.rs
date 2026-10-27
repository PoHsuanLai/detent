//! The card under the search field: the person's picture and name; a press opens Accounts.

use crate::live::use_client;
use crate::marked::marked;
use detent_model::Person;
use dioxus::prelude::*;
use ds::components::content::avatar::{
    AvatarFace, AvatarShape, AvatarSize, AvatarTone, person_hue,
};
use ds::components::lists::row::size::RowSize;
use ds::prelude::*;

/// The avatar's size; the picture is drawn at the same side (`.person-picture`).
const SIZE: AvatarSize = AvatarSize::Size34;

/// What leads the card: the person's picture when they have one, else a disc with their initial.
fn leading(person: &Person) -> RowLeading {
    let picture = person
        .picture
        .as_deref()
        .and_then(|path| ImageSource::file(path).ok());
    match picture {
        Some(source) => RowLeading::Element(rsx! {
            img { class: "person-picture", alt: "", src: "{source.0}", draggable: "false" }
        }),
        None => RowLeading::Avatar(AvatarFace {
            initial: person.initial(),
            size: SIZE,
            tone: AvatarTone::Person(person_hue(&person.name)),
            shape: AvatarShape::Round,
        }),
    }
}

/// The person using this computer, read once when the window opens; nothing is drawn while the
/// client does not know who they are. A press hands to `onopen`.
#[component]
pub(crate) fn PersonCard(onopen: EventHandler<()>) -> Element {
    let client = use_client();
    let mut person = use_signal(|| None::<Person>);
    use_future(move || {
        let client = client.clone();
        async move { person.set(client.whoami().person().await) }
    });
    let Some(person) = person() else {
        return rsx! {};
    };
    rsx! {
        Row {
            leading: leading(&person),
            title: TextLine::from(person.name.clone()),
            detail: Some(TextLine::from(crate::copy::window::PERSON_LINE)),
            size: RowSize::Settings,
            common: marked("person-card", &person.name),
            onclick: move |_| onopen.call(()),
        }
    }
}
