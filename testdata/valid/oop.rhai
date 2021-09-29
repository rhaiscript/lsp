// This script simulates object-oriented programming (OOP) techniques using closures.

// External variable that will be captured.
let last_value = ();

// Define object
let obj1 = #{
    _data: 42,                                  // data field
    get_data: || this._data,                    // property getter
    action: || print(`Data=${this._data}`),     // method
    update: |x| {                               // property setter
        this._data = x;
        last_value = this._data;                // capture 'last_value'
        this.action();
    }
};

if obj1.get_data() > 0 {                        // property access
    obj1.update(123);                           // call method
} else {
    print("we have a problem here");
}

// Define another object based on the first object
let obj2 = #{
    _data: 0,                                   // data field - new value
    update: |x| {                               // property setter - another function
        this._data = x * 2;
        last_value = this._data;                // capture 'last_value'
        this.action();
    }
};
obj2.fill_with(obj1);                           // add all other fields from obj1

if obj2.get_data() > 0 {                        // property access
    print("we have another problem here");
} else {
    obj2.update(42);                            // call method
}

print(`Should be 84: ${last_value}`);
