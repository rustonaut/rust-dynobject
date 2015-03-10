use std::result::Result;
use std::ops::{Index, IndexMut};
use std::collections::HashMap;
use std::hash::Hash;
use std::any::Any;

//import and reexport dyn_property
use super::dyn_property::DynProperty;


/// zero sized type used as "is undefined" marker
pub struct UndefinedProperty;

pub fn undefined_property() -> DynProperty {
    //pointer to zero sized Type -> any non zero pointer is ok ( test shows it uses 0x1 ) so no
    //allocation on heap is done
    DynProperty::new( Box::new( UndefinedProperty ))
}

/// The inner part of DynamicObject witch contains the data
///
/// This Trait provids a way to create, add, remove and
/// modify typed propertys defined by a given Key
///
pub struct InnerDynObject<Key> {
    //initialise this allways with DynProperty::undefined();
    //FIXME move this as assoziated Konstant ( with unsave ) or static
    undefined_property: DynProperty,
    data: HashMap<Key, DynProperty>
}

impl<Key> InnerDynObject<Key> where Key: Eq + Hash {
    
    /// Creates a new empty InnerDynObject
    ///
    pub fn new() -> InnerDynObject<Key> {
        InnerDynObject {
            undefined_property: undefined_property(),
            data: HashMap::<Key, DynProperty>::new()
        }
    }

    /// sets the property defined by key
    ///
    /// If the property identified by key exists and the property has the type given by
    /// `T` this methode will set the value as new value and will return the old value
    /// as Ok( Box( T )). If the property does not exists or the type is wrong the passed
    /// value will be returned as Err( Box( T ))
    ///
    /// This is mostly equivalent to using  `inner_dyn_object[key].set( value )`
    ///
    /// # Example
    /// 
    #[unstable( reason="redundant, might be removed" )]
    #[inline]
    pub fn set_property<T>( &mut self, key: &Key, value: Box<T> ) -> Result<Box<T>,Box<T>> 
        where T: Any + 'static 
    {
        self.index_mut( key ).set( value )
    }
    
    /// create a new property with a initial value
    ///
    /// Creaates a new property with given key and `initial_value setting` the type of
    /// the property to the type of `initial_value`. If the property already exists
    /// the given initialvalue will be returned as `Err( Box( T ))` else `Ok( () )` will
    /// be returned.
    ///
    pub fn create_property<T>( &mut self, key: Key, init_value: Box<T> ) -> Result<(),Box<T>> 
        where T: Any + 'static  
    {
        if self.data.contains_key( &key ) {
            Err( init_value )
        } else {
            self.data.insert( key, DynProperty::new( init_value ));
            Ok( () )
        }
    }
    
    /// removes a given property returning the old value of it
    ///
    /// If the property exists and the type match the old
    /// value will eb returned wraped in a Ok-Result and the
    /// property is removed. Else the property won't be changed
    /// and `Err( () )`. If you e.g. try to remove a given property
    /// not the using the right type the property will NOT be removed.
    ///
    pub fn remove_property<T>( &mut self, key: &Key ) -> Result<Box<T>, ()> 
        where T: Any + 'static
    {
        if !self.index( key ).is_inner_type::<T>() {
            return Err( () );
        }
        Ok( self.data.remove( key ).unwrap().destruct::<T>().unwrap() )
    }

    /// returns true if a given property exists
    pub fn exists_property( &self, key: &Key ) -> bool {
        self.data.contains_key( key )
    }

    /// returns true if a given property exists and has the given type
    pub fn exists_property_with_type<T>( &self, key: &Key ) -> bool 
        where T: Any + 'static 
    {
        self.index( key ).is_inner_type::<T>()
    }

    //TODO add remove_typeles to remove without knowing the type returning Box<Any>
}

impl<Key: Hash+Eq> Index<Key> for InnerDynObject<Key> {
    type Output = DynProperty;
 
    /// return a reference to a `DynProperty` for a given key
    ///
    /// If the key exists in this `InnerDynObject` a reference to
    /// the associated property will be returned. If not a reference to
    /// a property with the inner type `UndefinedProperty` will be returned.
    ///
    fn index<'a>( &'a self, index: &Key ) -> &'a DynProperty {
        match self.data.get( index ) {
            Some( data ) => data,
            None => &self.undefined_property
        }
    }
}

impl<Key: Hash+Eq> IndexMut<Key> for InnerDynObject<Key> {


    /// return a mutable referenc to a `DynProperty` for a given key
    ///
    /// If the key exists in this `InnerDynObject` a reference to
    /// the associated property will be returned. If not a reference to
    /// a property with the inner type `UndefinedProperty` will be returned.
    ///
    /// Note: that it is not a problem to return a mutable reference to a UndefinedProperty
    /// because UndefinedProperty is a zero sized type and therfor has only one representation.
    ///
    fn index_mut<'a>( &'a mut self, index: &Key ) -> &'a mut DynProperty {
        match self.data.get_mut( index ) {
            Some( data ) => data,
            None => &mut self.undefined_property
        }
    }
}

#[cfg( test )]
mod test {
    use super::InnerDynObject;
    use super::UndefinedProperty;
    use super::undefined_property;

    fn create_dummy() -> InnerDynObject<&'static str> {
        InnerDynObject::<&'static str>::new()
    }

    #[test]
    fn exist_property_should_return_false_if_inexisting() {
        let obj = create_dummy();
        assert!( !obj.exists_property( &"hallo" ) );
    }


    #[test]
    fn after_creating_a_property_should_exist() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        assert!( obj.exists_property( &"hallo" ));
    }


    #[test]
    fn exist_property_with_type_should_return_true_if_property_exists_and_has_given_type() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        assert!( obj.exists_property_with_type::<i32>( &"hallo" ));
    }

    #[test]
    fn exists_property_with_type_should_return_false_if_property_is_undefined() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        assert!( !obj.exists_property_with_type::<i32>( &"NOThallo" ));
    }

    #[test]
    fn exists_property_with_type_should_return_fals_if_the_type_mismatches() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        assert!( !obj.exists_property_with_type::<u16>( &"hallo" ));
    }

    #[test]
    fn create_property_should_return_true_if_key_is_new() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
    }

    

    #[test]
    fn create_property_should_return_false_if_key_already_exists() {
        let mut obj = create_dummy();
        assert!(  obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        assert!( !obj.create_property( "hallo", Box::new( 20i32 )).is_ok() );

    }

    #[test]
    fn set_property_should_err_if_property_does_not_exist() {
        let mut obj = create_dummy();
        assert!( !obj.exists_property( &"hallo" ) );
        let res = obj.set_property( &"hallo", Box::new( 44i32 ));
        assert_eq!( res, Err( Box::new( 44i32 )) );
    }

    #[test]
    fn set_property_should_err_if_property_type_is_wrong() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        let res = obj.set_property( &"hallo", Box::new( "oh falsch" ));
        assert_eq!( res, Err( Box::new( "oh falsch" )) );
    }
    
    #[test]
    fn set_property_should_return_the_old_value_if_property_exists_and_type_matches() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        let res = obj.set_property( &"hallo", Box::new( 44i32 ));
        assert_eq!( res, Ok( Box::new( 23i32 )) );
        //obj["hallo"] expands to obj.index( &"hallo" ) 
        assert_eq!( obj["hallo"].as_ref::<i32>().unwrap(), &44i32 );
    }

    #[test]
    fn remove_property_should_fail_if_property_does_not_exists() {
        let mut obj = create_dummy();
        let res = obj.remove_property::<i32>( &"hallo" );
        assert_eq!( res, Err( () ) );
    }

    #[test]
    fn remove_property_should_fail_if_the_type_mismatches() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        let res = obj.remove_property::<u16>( &"hallo" );
        assert_eq!( res, Err( () ) );
        assert!( obj.exists_property( &"hallo" ));
    }

    #[test]
    fn remove_property_should_return_the_property_value_if_succesful() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        let res = obj.remove_property::<i32>( &"hallo" );
        assert_eq!( res, Ok( Box::new( 23i32 )) );
    }

    //TODO test index
    #[test]
    fn index_should_return_the_property_if_existing() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        let ref res = obj["hallo"];
        assert!( res.is_inner_type::<i32>() );
        match res.as_ref::<i32>() {
            Some( ref_val ) => assert_eq!( ref_val, &23i32 ),
            None => panic!( "expected to be borrowable in this situration" )
        }   
    }

    #[test]
    fn index_should_return_a_property_of_type_undefined_if_inexisting() {
        let obj = create_dummy();
        assert!( obj["hallo"].is_inner_type::<UndefinedProperty>() );
    }

    #[test]
    fn index_should_also_allow_mutable_access() {
        let mut obj = create_dummy();
        assert!( obj.create_property( "hallo", Box::new( 23i32 )).is_ok() );
        match obj["hallo"].as_mut::<i32>() {
            Some( ref_val ) => {
                let mut dummy = 23i32;
                assert_eq!( ref_val, &mut dummy )
            }
            None => panic!( "expected to be borrowable in this situration" )
        }   
    }
    
    #[test]
    fn undefined_property_should_return_a_property_of_the_undefined_property_type() {
        let x = undefined_property();
        assert!( x.is_inner_type::<UndefinedProperty>() );
    }
}
