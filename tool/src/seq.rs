pub enum Seq<'a,T> {
    One(&'a T),
    Cat(&'a [&'a Self]),
}

impl<'a,T> Seq<'a,T> {
    pub fn iter(&self)->SeqIter<'a,T> {
	match self {
	    Self::One(x) => SeqIter {
		top:Some(x),
		stack:Vec::new()
	    },
	    Self::Cat(s) => SeqIter {
		top:None,
		stack:vec![s]
	    }
	}
    }
}

pub struct SeqIter<'a,T> {
    top:Option<&'a T>,
    stack:Vec<&'a [&'a Seq<'a,T>]>
}

impl<'a,T> Iterator for SeqIter<'a,T> {
    type Item = &'a T;
    
    fn next(&mut self)->Option<&'a T> {
	loop {
	    if let Some(x) = self.top.take() {
		return Some(x);
	    }
	    if let Some(s) = self.stack.pop() {
		match s {
		    [] => (),
		    [s,rest @ ..] => {
			match s {
			    Seq::One(x) => {
				self.stack.push(rest);
				return Some(x)
			    },
			    Seq::Cat(t) => {
				self.stack.push(rest);
				self.stack.push(t);
			    }
			}
		    }
		}
	    } else {
		return None
	    }
	}
    }
}

#[test]
fn test_seq() {
    let foo = Seq::Cat(&[
	&Seq::One(&"hello"),
	&Seq::One(&"world"),
	&Seq::Cat(&[
	    &Seq::One(&"this"),
	    &Seq::One(&"is"),
	    &Seq::One(&"a"),
	]),
	&Seq::One(&"test"),
	&Seq::Cat(&[
	    &Seq::One(&"of"),
	    &Seq::One(&"concatenation"),
	    &Seq::One(&"facilities"),
	]),
    ]);
    for x in foo.iter() {
	println!(">> {}",x);
    }
}
