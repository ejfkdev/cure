package japa.bdd.samples;

@Deprecated
public class JavaConceptsEnums {
    public enum Teste {
        asc, def
    }
    public enum Sexo {
        m, @Deprecated
        f;
        public enum Sexo_ implements Serializable, Cloneable {
        }
    }
    @Deprecated
    public enum Enum {
        m(1) {

            @Override
            void mm() {
            }
        }, f(2) {

            void mm() {
            }
        };
        native void nnn();
        transient int x;
        private Enum(int x) {
            this.x = x;
        }
        abstract void mm();
    }
}
