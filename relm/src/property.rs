use gtk::{glib::{self}, prelude::*, EntryBuffer, Frame, Label};
use libindigo::{PropertyItem as IndigoPropertyItem, PropertyType, PropertyValue, Property as IndigoProperty};
use relm4::{
    factory::{FactoryHashMap, FactoryVecDequeBuilder, FactoryView}, gtk, prelude::{DynamicIndex, FactoryComponent, FactoryVecDeque}, view, Component, ComponentParts, ComponentSender, FactorySender, RelmWidgetExt, SimpleComponent
};

pub(crate) struct Property {
    property: IndigoProperty,
    items: FactoryVecDeque<PropertyItem>,
}


#[derive(Debug, Clone)]
pub(crate) enum PropertyInput {
    UpdateProperty(IndigoProperty),
    DeleteProperty(IndigoProperty),
}

#[derive(Debug, Clone)]
pub(crate) enum PropertyOutput {
    UpdateItem(PropertyItem),
}

#[relm4::factory(pub)]
impl FactoryComponent for Property {
    type Init = IndigoProperty;
    type Input = PropertyInput;
    type Output = PropertyOutput;
    type CommandOutput = ();
    type ParentWidget = gtk::Box;
    type Index = String;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Vertical,
            set_spacing: 10,
            gtk::Frame {
                set_label: Some(self.property.label()),
                self.items.widget() ->
                &gtk::Box {
                    set_orientation: gtk::Orientation::Vertical,
                }
            }
            // TODO add property items from factory
        },
    }

    fn init_model(property: Self::Init, _index: &String, sender: FactorySender<Self>) -> Self {
        let items = FactoryVecDeque::builder()
            .launch(Self::ParentWidget::default())
            .detach();
            // .forward(sender.input_sender(), |output| match output {
            //     Prop => PropertyInput::,
            // });
        Self { property, items }
    }

    fn init_widgets(
        &mut self,
        _index: &Self::Index,
        root: Self::Root,
        returned_widget: &<Self::ParentWidget as FactoryView>::ReturnedWidget,
        sender: FactorySender<Self>,
    ) -> Self::Widgets {
        let widgets = view_output!();

        for item in self.property.items() {
            self.items.guard().push_back(item.clone());
        }

        widgets
    }

    fn update(&mut self, message: Self::Input, sender: FactorySender<Self>) {
        match message {
            PropertyInput::UpdateProperty(p) => todo!(),
            PropertyInput::DeleteProperty(p) => todo!(),
        }
    }

}

#[derive(Debug, Clone)]
struct PropertyItem {
    item: IndigoPropertyItem,
}

#[derive(Debug)]
pub enum PropertyItemInput {
    Toggle,
}

#[relm4::factory]
impl FactoryComponent for PropertyItem {
    type Init = IndigoPropertyItem;
    type Input = PropertyItemInput;
    type Output = ();
    type CommandOutput = ();
    type ParentWidget = gtk::Box;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,
            append = match &self.item.value {
                PropertyValue::Text(s) => {
                    gtk::Box {
                        set_orientation: gtk::Orientation::Horizontal,
                        set_spacing: 10,
                        gtk::Label {
                            #[watch]
                            set_label: &self.item.name,
                        },
                        gtk::Label {
                            #[watch]
                            set_label: s,
                        }
                    }
                }
                // PropertyValue::Number{format, min, max, step, value, target} => {
                PropertyValue::Number{..} => {
                    gtk::Label {
                        set_label: "TODO: render number property"
                    }
                }
                PropertyValue::Light(_) => {
                    gtk::Label {
                        set_label: "TODO: render light property"
                    }
                }
                PropertyValue::Switch(b) => {
                    gtk::CheckButton {
                        #[watch]
                        set_active: *b,
                        set_label: Some(&self.item.name),
                        // TODO group checkbuttons depending on the property SwitchType...
                        // #[track(true)]
                        // set_group: Some("apa"),
                    }
                }
                // PropertyValue::Blob{format, url, size, value} => {
                PropertyValue::Blob{..} => {
                    gtk::Label {
                        set_label: "TODO: render blob property"
                    }
                }
            },
        }

    }

    fn init_model(item: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { item }
    }
}

impl PropertyItem {
    fn label(&self) -> &str {
        &self.item.label
    }

    fn new(item: IndigoPropertyItem) -> Self {
        Self { item }
    }
}

struct SwitchItem {
    item: &'static IndigoPropertyItem,
}

#[derive(Debug)]
pub enum SwitchCommand {
    Toggle,
}

#[derive(Debug)]
pub enum SwitchStatus {
    On, Off
}

#[relm4::factory]
impl FactoryComponent for SwitchItem {
    type Init = &'static IndigoPropertyItem;
    type Input = SwitchCommand;
    type Output = SwitchStatus;
    type CommandOutput = ();
    type ParentWidget = gtk::Stack;

    view! {
        #[root]
        gtk::Box {
            set_orientation: gtk::Orientation::Horizontal,
            set_spacing: 10,
            gtk::Switch {
                #[watch]
                set_active: self.value(),
            },
            gtk::Label {
                #[watch]
                set_label: &self.label(),
            }
        }
    }

    fn init_model(item: Self::Init, _index: &DynamicIndex, _sender: FactorySender<Self>) -> Self {
        Self { item }
    }

}

impl SwitchItem {
    fn label(&self) -> &str {
        &self.item.label
    }
    fn value(&self) -> bool {
        if let PropertyValue::Switch(b) = self.item.value {
            b
        } else {
            unreachable!("expected 'PropertyStatus::Switch' found '{}'", &self.item.value)
        }
    }
}