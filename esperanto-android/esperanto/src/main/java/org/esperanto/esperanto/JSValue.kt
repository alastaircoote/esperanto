package org.esperanto.esperanto

private object JSValuePrivate {
    @JvmStatic external fun asNumber(val_ptr: Long) : Double;
    @JvmStatic external fun asString(val_ptr: Long) : String;
    @JvmStatic external fun getProperty(val_ptr: Long, name: String) : Long;
    @JvmStatic external fun call(val_ptr: Long, bound_to: Long) : Long;
//    @JvmStatic external fun evaluate(ctx_ptr: Long) : Long;

}

class JSValue(private val val_ptr: Long) {

    fun asNumber() : Double {
        return JSValuePrivate.asNumber(val_ptr)
    }

    fun getProperty(name: String) : JSValue {
        val ptr = JSValuePrivate.getProperty(val_ptr, name)
        return JSValue(ptr)
    }

    fun call(boundTo: JSValue) : JSValue {
        val ptr = JSValuePrivate.call(val_ptr, boundTo.val_ptr);
        return JSValue((ptr))
    }

    fun asString() : String {
        return JSValuePrivate.asString(val_ptr);
    }
}