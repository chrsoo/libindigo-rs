use dtd::dtd;

dtd! {
    "<!ELEMENT note (to,from,heading,body)>
    <!ELEMENT to (#PCDATA)>
    <!ELEMENT from (#PCDATA)>
    <!ELEMENT heading (#PCDATA)>
    <!ELEMENT body (#PCDATA)>"
}

fn test() {
    let note = Note {
        to: todo!(),
        from: todo!(),
        heading: todo!(),
        body: todo!(),
    };
}