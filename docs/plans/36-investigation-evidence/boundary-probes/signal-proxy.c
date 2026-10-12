#define PY_SSIZE_T_CLEAN
#include <Python.h>
#include <time.h>
#include <unistd.h>
#include <errno.h>
struct ArrowArrayStream { int (*get_schema)(struct ArrowArrayStream*,void*); int (*get_next)(struct ArrowArrayStream*,void*); const char *(*get_last_error)(struct ArrowArrayStream*); void (*release)(struct ArrowArrayStream*); void *private_data; };
static int (*original_next)(struct ArrowArrayStream*,void*);
static int entered_fd, settled_fd;
static double duration;
static int seen;
static double now(void) {struct timespec t;clock_gettime(CLOCK_MONOTONIC,&t);return t.tv_sec+t.tv_nsec/1e9;}
static void pause_for(double seconds) {double end=now()+seconds;while(now()<end) {double rem=end-now();struct timespec t={(time_t)rem,(long)((rem-(time_t)rem)*1e9)};nanosleep(&t,0);}}
static void mark(int fd,const char *p) {if(write(fd,p,1)!=1)_exit(99);}
static int delayed_next(struct ArrowArrayStream *s,void *a) {if(seen++==0)return original_next(s,a);mark(entered_fd,"E");pause_for(duration);int result=original_next(s,a);mark(settled_fd,"S");return result;}
static PyObject *install(PyObject *self,PyObject *args) {PyObject *capsule;if(!PyArg_ParseTuple(args,"Oiid",&capsule,&entered_fd,&settled_fd,&duration))return NULL;struct ArrowArrayStream *stream=PyCapsule_GetPointer(capsule,"arrow_array_stream");if(!stream)return NULL;seen=0;original_next=stream->get_next;stream->get_next=delayed_next;Py_RETURN_NONE;}
static PyObject *bridge(PyObject *self,PyObject *args) {int e,c,s;double total;if(!PyArg_ParseTuple(args,"iiid",&e,&c,&s,&total))return NULL;mark(e,"E");double end=now()+total;PyObject *type=NULL,*value=NULL,*trace=NULL;while(now()<end) {Py_BEGIN_ALLOW_THREADS pause_for(.1);Py_END_ALLOW_THREADS if(!type && PyErr_CheckSignals()<0){PyErr_Fetch(&type,&value,&trace);mark(c,"C");end=now()+.2;}}mark(s,"S");if(type){PyErr_Restore(type,value,trace);return NULL;}Py_RETURN_NONE;}
static PyMethodDef methods[]={{"install",install,METH_VARARGS,NULL},{"bridge",bridge,METH_VARARGS,NULL},{NULL,NULL,0,NULL}};
static struct PyModuleDef module={PyModuleDef_HEAD_INIT,"signal_proxy",NULL,-1,methods};
PyMODINIT_FUNC PyInit_signal_proxy(void){return PyModule_Create(&module);}
