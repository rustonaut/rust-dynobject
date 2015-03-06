use std::result::Result;
use std::mem;
use std::any::Any;
use std::boxed::BoxAny;

pub struct UndefinedProperty;

pub struct DynProperty {
    value: Box<Any+'static>
}

impl DynProperty {

    pub fn new<T: Any>(initial_value: Box<T>) -> DynProperty {
        DynProperty { value: initial_value }
    }
    
    pub fn undefined() -> DynProperty {
        //pointer to zero sized Type -> any non zero pointer is ok (test shows it uses 0x1) so no
        //allocation on heap is done
        let irrelevant_box = Box::new(UndefinedProperty);
        DynProperty::new(irrelevant_box)
    }

    //set new return old or None if type mismatch
    pub fn set<T>(&mut self, value: Box<T>) -> Result<Box<T>,Box<T>> 
        where T: Any+'static
    {
        if self.is_inner_type::<T>() {
            let mut any_boxed = value as Box<Any+'static>;
            mem::swap(&mut any_boxed, &mut self.value);
            Ok(any_boxed.downcast::<T>().unwrap())
        } else {
            Err(value)
        }   
            
    }

    pub fn as_ref<'a, T: Any + 'static>(&'a self) -> Option<&'a T> {
        self.value.downcast_ref()
    }

    pub fn as_mut<'a, T: Any>(&'a mut self) -> Option<&'a mut T> {
        self.value.downcast_mut::<T>()
    }
    
    pub fn is_inner_type<T:Any>(&self) -> bool {
        self.value.is::<T>()
    }

    pub fn destruct<T:Any>(self) -> Option<Box<T>> where T: 'static {
        if self.is_inner_type::<T>() {
            Some(self.value.downcast::<T>().unwrap())
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::DynProperty;
    use super::UndefinedProperty;

    //a simple data Type
    #[derive(Eq, PartialEq, Debug)]
    struct Point(i32,i32);

    #[derive(Eq, PartialEq, Debug)]
    struct Point3D(i32,i32,i32);

    fn first_dummy_value() -> i32 { 12 }
    fn second_dummy_value() -> i32 { 25 }
    fn dummy_value() -> Point {
        Point(first_dummy_value(), second_dummy_value())
    }
    fn box_dummy_value() -> Box<Point> {
        Box::new(dummy_value())
    }

    fn create_dummy() -> DynProperty {
        DynProperty::new::<Point>(box_dummy_value())
    }

    #[test]
    fn is_inner_type_should_return_false_if_type_differs() {
        let x = create_dummy();
        assert!(x.is_inner_type::<Point>());
    }

    #[test]
    fn is_inner_type_should_return_true_if_type_is_same() {
        let x = create_dummy();
        assert!(!x.is_inner_type::<i32>());
    }

    #[test]
    fn destruct_shuld_return_inner_value_if_type_matchs() {
        let x = create_dummy();
        assert_eq!(x.destruct::<Point>(), Some(box_dummy_value()));
    }
    
    #[test]
    fn if_destructor_inner_type_mismatched_inner_value_should_also_be_destructed() {
        use std::rc;
        use std::rc::Rc;
        let load = Rc::new(5i32);
        let other_load = load.clone();
        assert!( !rc::is_unique(&other_load) );
        let dyn_prop = DynProperty::new::<Rc<i32>>(Box::new(load));
        //intentionally wrong type
        let res = dyn_prop.destruct::<Point>();
        assert_eq!(res, None);
        assert!( rc::is_unique(&other_load) )
    }

    #[test]
    fn as_ref_should_return_some_if_type_matchs() {
        let x = create_dummy();
        let inner = x.as_ref::<Point>().unwrap();
        let expected = dummy_value();
        assert_eq!(inner, &expected);
    }

    #[test]
    fn as_ref_should_return_none_if_type_mismatches() {
        let x = create_dummy();
        assert_eq!(x.as_ref::<i32>(), None);
    }

    #[test]
    fn as_mut_should_return_some_if_type_matches() {
        let mut x = create_dummy();
        let inner = x.as_mut::<Point>().unwrap();
        let mut expected = dummy_value();
        assert_eq!(inner, &mut expected);
        
    }
    
    #[test]
    fn as_mut_should_allow_mutating_the_inner_type() {
        let modif = 10;
        let mut x = create_dummy();
        match x.as_mut::<Point>() {
            Some(rpoint) => rpoint.0 += modif,
            None => panic!("as_mut did return None, where it shouldn't")
        }
        match x.as_ref::<Point>() {
            Some(rpoint) => assert_eq!(rpoint.0, first_dummy_value()+modif),
            None => panic!("as_ref did return None, where it shouldn't")
        }
    }

    #[test]
    fn as_mut_should_return_none_if_type_mismatches() {
        let mut x = create_dummy();
        assert_eq!(x.as_mut::<i32>(), None);
    }

    #[test]
    fn set_should_return_ok_if_old_value_if_type_matches() {
        let mut x = create_dummy();
        let res = x.set(Box::new(Point(second_dummy_value(), first_dummy_value())));
        assert_eq!(res, Ok(box_dummy_value()));
        assert_eq!(x.destruct::<Point>(), Some(Box::new(Point(second_dummy_value(), first_dummy_value()))));
    } 

    #[test]
    fn set_should_return_err_of_the_parameter_if_type_mismatches() {
        let mut x = create_dummy();
        let res = x.set(Box::new(Point3D(1,1,1)));
        assert_eq!(res, Err(Box::new(Point3D(1,1,1))));    
    }

    #[test]
    fn undefined_should_return_a_property_of_the_undefined_property_type() {
        let x = DynProperty::undefined();
        assert!(x.is_inner_type::<UndefinedProperty>());
    }

}
