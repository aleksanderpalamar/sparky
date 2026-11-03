#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    LockAsked,
    UnlockAsked,
    Sleeping,
    Woke,
    Locked,
    Unlocked,
    Expired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Open,
    Locked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Order {
    Lock,
    Unlock,
    Hold,
    Release,
    Watch,
    Hint(Screen),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Sleep {
    Awake,
    Pending,
    Asleep,
}

pub struct Guard {
    screen: Screen,
    sleep: Sleep,
}

impl Guard {
    pub fn start() -> (Self, Vec<Order>) {
        let guard = Self {
            screen: Screen::Open,
            sleep: Sleep::Awake,
        };
        (guard, vec![Order::Hold, Order::Hint(Screen::Open)])
    }

    pub fn handle(&mut self, input: Input) -> Vec<Order> {
        match input {
            Input::LockAsked => vec![Order::Lock],
            Input::UnlockAsked => vec![Order::Unlock],
            Input::Sleeping => self.sleeping(),
            Input::Woke => self.woke(),
            Input::Locked => self.locked(),
            Input::Unlocked => {
                self.screen = Screen::Open;
                vec![Order::Hint(Screen::Open)]
            }
            Input::Expired => self.expired(),
        }
    }

    fn sleeping(&mut self) -> Vec<Order> {
        match (self.sleep, self.screen) {
            (Sleep::Awake, Screen::Locked) => {
                self.sleep = Sleep::Asleep;
                vec![Order::Release]
            }
            (Sleep::Awake, Screen::Open) => {
                self.sleep = Sleep::Pending;
                vec![Order::Lock, Order::Watch]
            }
            (Sleep::Pending | Sleep::Asleep, _) => Vec::new(),
        }
    }

    fn woke(&mut self) -> Vec<Order> {
        match self.sleep {
            Sleep::Awake => Vec::new(),
            Sleep::Pending | Sleep::Asleep => {
                self.sleep = Sleep::Awake;
                vec![Order::Hold]
            }
        }
    }

    fn locked(&mut self) -> Vec<Order> {
        self.screen = Screen::Locked;
        let mut orders = vec![Order::Hint(Screen::Locked)];
        orders.extend(self.expired());
        orders
    }

    fn expired(&mut self) -> Vec<Order> {
        match self.sleep {
            Sleep::Pending => {
                self.sleep = Sleep::Asleep;
                vec![Order::Release]
            }
            Sleep::Awake | Sleep::Asleep => Vec::new(),
        }
    }
}
