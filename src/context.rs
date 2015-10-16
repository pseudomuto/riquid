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
    variables: Vec<HashMap<String, Variable>>,
    current_scope: usize,
}

impl Context {
    pub fn new() -> Context {
        let variables = HashMap::new();
        Context { variables: vec![variables], current_scope: 0 }
    }

    fn current_variables(&mut self) -> &mut HashMap<String, Variable> {
        &mut self.variables[self.current_scope]
    }

    pub fn add<T: Any>(&mut self, key: &str, val: &T) {
        let val_any = val as &Any;

        if let Some(string) = val_any.downcast_ref::<&str>() {
            self.current_variables().insert(key.to_string(), Variable::String(string.to_string()));
        } else if let Some(number) = val_any.downcast_ref::<f64>() {
            self.current_variables().insert(key.to_string(), Variable::Number(*number));
        } else if let Some(boolean) = val_any.downcast_ref::<bool>() {
            self.current_variables().insert(key.to_string(), Variable::Boolean(*boolean));
        } else {
            panic!("Tried to add unknown type to context");
        }

    }

    pub fn lookup(&mut self, key: &str) -> Option<&Variable> {
        for scope in self.variables.iter().rev() {
            match scope.get(key) {
                Some(val)   => return Some(val),
                None        => continue,
            }
        }
        None
    }

    pub fn push(&mut self) {
        self.variables.push(HashMap::new());
        self.current_scope += 1;
    }

    pub fn pop(&mut self) {
        if self.current_scope >= 1 {
            self.variables.pop();
            self.current_scope -= 1;
        } else {
            panic!("tried to pop one too many scopes!");
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_works() {
        let context = Context::new();
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

    #[test]
    fn can_push_new_scope() {
        let mut context = Context::new();
        context.push();
    }

    #[test]
    fn can_push_then_pop_scope() {
        let mut context = Context::new();
        context.push();
        context.pop();

    }

    #[test]
    #[should_panic(expected = "tried to pop one too many scopes!")]
    fn cant_pop_when_no_push() {
        let mut context = Context::new();
        context.pop();
    }

    #[test]
    fn lookup_current_scope() {
        let mut context = Context::new();
        context.push();
        context.add("test", &true);

        assert_eq!(*context.lookup("test").unwrap(), Variable::Boolean(true));
    }

    #[test]
    fn lookup_all_scopes() {
        let mut context = Context::new();
        context.add("test", &false);
        context.push();

        assert_eq!(*context.lookup("test").unwrap(), Variable::Boolean(false));
    }

    fn pop_clear_scope() {
        let mut context = Context::new();
        context.push();
        context.add("test", &true);
        context.pop();

        assert_eq!(context.lookup("test"), None);

    }

}
