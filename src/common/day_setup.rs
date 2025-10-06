use std::borrow::Cow;
use std::ops::Deref;

pub type InputProvider = dyn Fn() -> Cow<'static, str>;

enum InputFun {
    WithStr(fn(&str)),
    WithContext(fn(&AppContext)),
}

pub struct Day {
    fun: InputFun,
    test_inputs: Option<&'static [&'static str]>,
}

impl Day {
    pub fn new(fun: fn(&str)) -> Self {
        Self {
            fun: InputFun::WithStr(fun),
            test_inputs: None,
        }
    }
    pub fn custom(fun: fn(&AppContext)) -> Self {
        Self {
            fun: InputFun::WithContext(fun),
            test_inputs: None,
        }
    }
    pub fn with_test_inputs(mut self, test_inputs: &'static [&'static str]) -> Self {
        self.test_inputs = Some(test_inputs);
        self
    }
    pub fn exec(self, context: &mut AppContext) {
        context.add_test_inputs(
            self.test_inputs
                .unwrap_or_default()
                .iter()
                .map(|&input| Box::new(move || input.into()) as Box<InputProvider>),
        );
        match self.fun {
            InputFun::WithStr(fun) => {
                fun(context.get_input().as_str());
            }
            InputFun::WithContext(fun) => {
                fun(context);
            }
        }
    }
}

#[derive(Default)]
pub struct AppContext {
    testing: Option<usize>,
    text_input: Option<Box<InputProvider>>,
    testing_inputs: Vec<Box<InputProvider>>,
}

impl AppContext {
    pub fn set_testing(&mut self, testing: Option<usize>) {
        self.testing = testing;
    }
    pub fn set_text_input(&mut self, text_input: Box<InputProvider>) {
        self.text_input = Some(text_input);
    }
    pub fn add_test_inputs(&mut self, test_input: impl Iterator<Item = Box<InputProvider>>) {
        if self.testing.is_none() {
            return;
        }
        self.testing_inputs.extend(test_input);
    }
    pub fn is_testing(&self) -> bool {
        self.testing.is_some()
    }

    pub fn get_input(&self) -> TextInput {
        if let Some(testing) = self.testing {
            return self.testing_inputs[testing]().into();
        }
        if let Some(text_input) = &self.text_input {
            return text_input().into();
        }
        panic!("No input provider set");
    }
}

pub struct TextInput(Cow<'static, str>);

impl TextInput {
    pub fn as_str(&self) -> &str {
        clean_input(self.0.as_ref())
    }
}

impl From<Cow<'static, str>> for TextInput {
    fn from(value: Cow<'static, str>) -> Self {
        Self(value)
    }
}

impl From<&'static str> for TextInput {
    fn from(value: &'static str) -> Self {
        Self(Cow::Borrowed(value))
    }
}

impl From<String> for TextInput {
    fn from(value: String) -> Self {
        Self(Cow::Owned(value))
    }
}

impl AsRef<str> for TextInput {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Deref for TextInput {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

fn clean_input(input: &str) -> &str {
    input.trim().trim_start_matches('\u{feff}')
}
