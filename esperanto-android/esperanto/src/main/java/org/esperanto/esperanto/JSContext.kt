package org.esperanto.esperanto

private object JSContextPrivate {
    @JvmStatic external fun new() : Long;
    @JvmStatic external fun evaluate(ctx_ptr: Long, script:String) : Long;

    init {
        System.loadLibrary("esperanto")
    }
}

class JSContext {

    private var ptr:Long = JSContextPrivate.new()

    fun evaluate(script:String) : JSValue {
        val result_ptr = JSContextPrivate.evaluate(ptr, script)
        return JSValue(result_ptr)
    }


    external fun instanceTest() : String;

}