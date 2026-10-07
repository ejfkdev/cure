import java.util.List;
import java.util.ArrayList;
import java.util.HashMap;
import java.lang.annotation.*;

class Top {
    @Target(ElementType.TYPE_USE) @interface TA {
    }
    @Target(ElementType.TYPE_USE) @interface TB {
    }
    @Target(ElementType.TYPE_USE) @interface TC {
    }
    static class Outer {
        class Inner {
            Object o2 = Outer.this;
            Object o3 = this;
        }
        static class SInner {
            Object o3 = this;
        }
        interface IInner {
        }
    }
    @TB Outer f1;
    @TB Outer.Inner f1a;
    @TB Outer.SInner f2a;
    @TB Outer.IInner f3a;
    Outer.Inner f1b;
    Outer.SInner f2b;
    Outer.IInner f3b;
    @TB Outer.Inner f1c;
    @TB Outer.SInner f2c;
    @TB Outer.IInner f3c;
    @TA Top.Outer g1;
    @TA Top.Outer.Inner g1a;
    @TA Top.Outer.SInner g2a;
    @TA Top.Outer.IInner g3a;
    @TA Top.Outer.Inner g1b;
    @TA Top.Outer.SInner g2b;
    @TA Top.Outer.IInner g3b;
    @TA Top.Outer.Inner g1c;
    @TA Top.Outer.SInner g2c;
    @TA Top.Outer.IInner g3c;
    @TB Outer f1r() {
        return null;
    }
    @TB Outer.Inner f1ra() {
        return null;
    }
    @TB Outer.SInner f2ra() {
        return null;
    }
    @TB Outer.IInner f3ra() {
        return null;
    }
    Outer.Inner f1rb() {
        return null;
    }
    Outer.SInner f2rb() {
        return null;
    }
    Outer.IInner f3rb() {
        return null;
    }
    @TB Outer.Inner f1rc() {
        return null;
    }
    @TB Outer.SInner f2rc() {
        return null;
    }
    @TB Outer.IInner f3rc() {
        return null;
    }
    void f1param(@TB Outer p, @TB Outer.Inner p1, Outer.Inner p2, @TB Outer.Inner p3) {}
    void f2param(@TB Outer p, @TB Outer.SInner p1, Outer.SInner p2, @TB Outer.SInner p3) {}
    void f3param(@TB Outer p, @TB Outer.IInner p1, Outer.IInner p2, @TB Outer.IInner p3) {}
    void f1cast(Object o) {
        Object l = (Outer) o;
        l = (Outer.Inner) o;
        l = (Outer.Inner) o;
        l = (Outer.Inner) o;
    }
    void f2cast(Object o) {
        Object l = (Outer) o;
        l = (Outer.SInner) o;
        l = (Outer.SInner) o;
        l = (Outer.SInner) o;
    }
    void f3cast(Object o) {
        Object l = (Outer) o;
        l = (Outer.IInner) o;
        l = (Outer.IInner) o;
        l = (Outer.IInner) o;
    }
    List<@TB Outer> h1;
    List<@TB Outer.Inner> h1a;
    List<@TB Outer.SInner> h2a;
    List<@TB Outer.IInner> h3a;
    List<Outer. @TC Inner> h1b;
    List<Outer. @TC SInner> h2b;
    List<Outer. @TC IInner> h3b;
    List<@TB Outer. @TC Inner> h1c;
    List<@TB Outer. @TC SInner> h2c;
    List<@TB Outer. @TC IInner> h3c;
    List<@TA Top. @TB Outer> k1;
    List<@TA Top. @TB Outer.Inner> k1a;
    List<@TA Top. @TB Outer.SInner> k2a;
    List<@TA Top. @TB Outer.IInner> k3a;
    List<@TA Top. Outer. @TC Inner> k1b;
    List<@TA Top. Outer. @TC SInner> k2b;
    List<@TA Top. Outer. @TC IInner> k3b;
    List<@TA Top. @TB Outer. @TC Inner> k1c;
    List<@TA Top. @TB Outer. @TC SInner> k2c;
    List<@TA Top. @TB Outer. @TC IInner> k3c;
    List<@TB Outer> g1r() {
        return null;
    }
    List<@TB Outer.Inner> g1ra() {
        return null;
    }
    List<@TB Outer.SInner> g2ra() {
        return null;
    }
    List<@TB Outer.IInner> g3ra() {
        return null;
    }
    List<Outer. @TC Inner> g1rb() {
        return null;
    }
    List<Outer. @TC SInner> g2rb() {
        return null;
    }
    List<Outer. @TC IInner> g3rb() {
        return null;
    }
    List<@TB Outer. @TC Inner> g1rc() {
        return null;
    }
    List<@TB Outer. @TC SInner> g2rc() {
        return null;
    }
    List<@TB Outer. @TC IInner> g3rc() {
        return null;
    }
    void g1param(List<@TB Outer> p, List<@TB Outer.Inner> p1, List<Outer. @TC Inner> p2, List<@TB Outer. @TC Inner> p3) {}
    void g2param(List<@TB Outer> p, List<@TB Outer.SInner> p1, List<Outer. @TC SInner> p2, List<@TB Outer. @TC SInner> p3) {}
    void g3param(List<@TB Outer> p, List<@TB Outer.IInner> p1, List<Outer. @TC IInner> p2, List<@TB Outer. @TC IInner> p3) {}
    void g1new(Object o) {
        Object l;
        l = new @TB ArrayList<@TB Outer>();
        l = new @TB ArrayList<@TB Outer.Inner>();
        l = new @TB HashMap<String, Outer. @TC Inner>();
        l = new @TB HashMap<String, @TB Outer. Inner>();
        l = new @TB HashMap<String, @TB Outer. @TC Inner>();
    }
    void g2new(Object o) {
        Object l;
        l = new @TB ArrayList<@TB Outer>();
        l = new @TB ArrayList<@TB Outer.SInner>(); // err
        l = new @TB HashMap<String, Outer. @TC SInner>();
        l = new @TB HashMap<String, @TB Outer. SInner>(); // err
        l = new @TB HashMap<String, @TB Outer. @TC SInner>(); // err
    }
    void g3new(Object o) {
        Object l;
        l = new @TB ArrayList<@TB Outer>();
        l = new @TB ArrayList<@TB Outer.IInner>(); // err
        l = new @TB HashMap<String, Outer. @TC IInner>();
        l = new @TB HashMap<String, @TB Outer. IInner>(); // err
        l = new @TB HashMap<String, @TB Outer. @TC IInner>(); // err
    }
    void g4new(Object o) {
        Object l;
        l = new @TB ArrayList<@TA Top. @TB Outer>(); // err
        l = new @TB ArrayList<@TA Top. @TB Outer.IInner>(); // err
        l = new @TB HashMap<String, @TA Top. Outer. @TC IInner>(); // err
        l = new @TB HashMap<String, @TA Top. @TB Outer. IInner>(); // err
        l = new @TB HashMap<String, @TA Top. @TB Outer. @TC IInner>(); // err
        l = new @TB HashMap<String, @TA @TB @TC Top. Outer. IInner>(); // err
    }
}
