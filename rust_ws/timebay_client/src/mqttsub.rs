use std::any::Any;
use iced::Subscription;
use crate::app::AppMessage;

pub fn mqtt_subscription() -> Subscription<AppMessage>{
    struct MqttWorker;

    iced::subscription::unfold(MqttWorker.type_id(),)
}