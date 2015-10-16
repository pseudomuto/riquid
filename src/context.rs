use std::collections::HashMap;
use std::any::Any;

#[derive(Debug,Clone,PartialEq)]
pub enum Variable {
    String(String),
    Number(f64),
    Boolean(bool),
    //other(Drop), coming soon(tm)
}

pub struct Context {
    variables: HashMap<String, Variable>,
}

impl Context {
    fn new() -> Context {
        let variables = HashMap::new();
        Context { variables: variables }
    }

    fn add<T: Any>(&mut self, key: &str, val: &T) {
        let val_any = val as &Any;

        if let Some(string) = val_any.downcast_ref::<&str>() {
            self.variables.insert(key.to_string(), Variable::String(string.to_string()));
        } else if let Some(number) = val_any.downcast_ref::<f64>() {
            self.variables.insert(key.to_string(), Variable::Number(*number));
        } else if let Some(boolean) = val_any.downcast_ref::<bool>() {
            self.variables.insert(key.to_string(), Variable::Boolean(*boolean));
        } else {
            panic!("Tried to add unknown type to context");
        }

    }

    fn lookup(&self, key: &str) -> Option<&Variable> {
        self.variables.get(key)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let mut context = Context::new();
    }

    #[test]
    fn lookup_missing_val() {
        let mut context = Context::new();
        assert_eq!(None, context.lookup("testt"));
    }

    #[test]
    fn add_string() {
        let mut context = Context::new();
        context.add("butt", &"face");
    }

    #[test]
    fn add_and_lookup_string() {
        let mut context = Context::new();
        context.add("butt", &"face");
        assert_eq!(*context.lookup("butt").unwrap(), Variable::String("face".to_string()));
    }

    #[test]
    fn add_number() {
        let mut context = Context::new();
        context.add("woop", &123.0f64);
    }

    #[test]
    fn add_and_lookup_number() {
        let mut context = Context::new();
        context.add("whoop", &123.0);
        assert_eq!(*context.lookup("whoop").unwrap(), Variable::Number(123.0));
    }

    #[test]
    fn add_boolean() {
        let mut context = Context::new();
        context.add("boolean", &true);
    }

    #[test]
    fn add_boolean_and_lookup() {
        let mut context = Context::new();
        context.add("boolean", &false);
        assert_eq!(*context.lookup("boolean").unwrap(), Variable::Boolean(false));
    }

    #[test]
    #[should_panic(expected = "Tried to add unknown type to context")]
    fn add_incompatible_type() {
        let mut context = Context::new();
        context.add("boom", &123);
    }
}
