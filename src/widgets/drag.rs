use glutin;

use ui;
use event::Target;
use widget::{WidgetBuilder, EventArgs};
use widget::WidgetBuilderCore;
use input::mouse::{MouseMoved, MouseButton, WidgetMouseButton};
use resources::WidgetId;
use util::Point;

pub struct WidgetDrag {
    pub drag_type: DragEvent,
    pub position: Point,
}

#[derive(Debug)]
pub enum DragEvent {
    DragStart,
    Drag,
    DragEnd,
}

pub struct DragInputHandler {
    widget: Option<WidgetId>,
    position: Point,
}
impl DragInputHandler {
    pub fn new() -> Self {
        DragInputHandler {
            widget: None,
            position: Point::new(0.0, 0.0),
        }
    }
}
impl ui::EventHandler<DragInputEvent> for DragInputHandler {
    fn handle(&mut self, event: &DragInputEvent, args: ui::EventArgs) {
        match *event {
            DragInputEvent::WidgetPressed(id) => {
                self.widget = Some(id);
                let event = WidgetDrag {
                    drag_type: DragEvent::DragStart,
                    position: self.position,
                };
                args.queue.push(Target::Widget(id), event);
            }
            DragInputEvent::MouseReleased => {
                if let Some(id) = self.widget {
                    self.widget = None;
                    let event = WidgetDrag {
                        drag_type: DragEvent::DragEnd,
                        position: self.position,
                    };
                    args.queue.push(Target::Widget(id), event);
                }
            }
            DragInputEvent::MouseMoved(point) => {
                self.position = point;
                if let Some(id) = self.widget {
                    let event = WidgetDrag {
                        drag_type: DragEvent::Drag,
                        position: self.position,
                    };
                    args.queue.push(Target::Widget(id), event);
                }
            }
        }
    }
}

pub enum DragInputEvent {
    WidgetPressed(WidgetId),
    MouseMoved(Point),
    MouseReleased,
}

fn drag_handle_mouse_press(event: &WidgetMouseButton, args: EventArgs) {
    if let &WidgetMouseButton(glutin::ElementState::Pressed, _) = event {
        let event = DragInputEvent::WidgetPressed(args.widget.id);
        args.queue.push(Target::Ui, event);
    }
}
pub fn drag_handle_mouse_move(event: &MouseMoved, args: ui::EventArgs) {
    args.queue.push(Target::Ui, DragInputEvent::MouseMoved(event.0));
}
pub fn drag_handle_mouse_release(event: &MouseButton, args: ui::EventArgs) {
    if let &MouseButton(glutin::ElementState::Released, _) = event {
        args.queue.push(Target::Ui, DragInputEvent::MouseReleased);
    }
}

impl WidgetBuilder {
    pub fn make_draggable(&mut self) -> &mut Self {
        self.as_mut().add_handler_fn(drag_handle_mouse_press);
        self
    }
}