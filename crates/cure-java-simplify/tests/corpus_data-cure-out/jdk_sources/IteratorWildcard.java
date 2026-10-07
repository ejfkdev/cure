package javax.security.auth;

import java.util.*;
import java.io.*;
import java.lang.reflect.*;
import java.text.MessageFormat;
import java.security.AccessController;
import java.security.AccessControlContext;
import java.security.DomainCombiner;
import java.security.Permission;
import java.security.PermissionCollection;
import java.security.Principal;
import java.security.PrivilegedAction;
import java.security.PrivilegedExceptionAction;
import java.security.PrivilegedActionException;
import java.security.ProtectionDomain;
import sun.security.util.ResourcesMgr;

public final class Subject implements java.io.Serializable {
    private static final long serialVersionUID = -8308522755600156056L;
    Set<Principal> principals;
    transient Set<Object> pubCredentials;
    transient Set<Object> privCredentials;
    private volatile boolean readOnly = false;
    private static final int PRINCIPAL_SET = 1;
    private static final int PUB_CREDENTIAL_SET = 2;
    private static final int PRIV_CREDENTIAL_SET = 3;
    private static final ProtectionDomain[] NULL_PD_ARRAY = new ProtectionDomain[0];
    public Subject() {
        this.principals = Collections.synchronizedSet(new SecureSet<>(this, PRINCIPAL_SET));
        this.pubCredentials = Collections.synchronizedSet(new SecureSet<>(this, PUB_CREDENTIAL_SET));
        this.privCredentials = Collections.synchronizedSet(new SecureSet<>(this, PRIV_CREDENTIAL_SET));
    }
    public Subject(boolean readOnly, Set<? extends Principal> principals, Set<?> pubCredentials, Set<?> privCredentials) {
        collectionNullClean(principals);
        collectionNullClean(pubCredentials);
        collectionNullClean(privCredentials);
        this.principals = Collections.synchronizedSet(new SecureSet<>(this, PRINCIPAL_SET, principals));
        this.pubCredentials = Collections.synchronizedSet(new SecureSet<>(this, PUB_CREDENTIAL_SET, pubCredentials));
        this.privCredentials = Collections.synchronizedSet(new SecureSet<>(this, PRIV_CREDENTIAL_SET, privCredentials));
        this.readOnly = readOnly;
    }
    public void setReadOnly() {
        java.lang.SecurityManager sm = System.getSecurityManager();
        if (sm != null) {
            sm.checkPermission(AuthPermissionHolder.SET_READ_ONLY_PERMISSION);
        }
        this.readOnly = true;
    }
    public boolean isReadOnly() {
        return this.readOnly;
    }
    public static Subject getSubject(final AccessControlContext acc) {
        java.lang.SecurityManager sm = System.getSecurityManager();
        if (sm != null) {
            sm.checkPermission(AuthPermissionHolder.GET_SUBJECT_PERMISSION);
        }
        Objects.requireNonNull(acc, ResourcesMgr.getString("invalid.null.AccessControlContext.provided"));
        return AccessController.doPrivileged(new java.security.PrivilegedAction<>() {
            public Subject run() {
                DomainCombiner dc = acc.getDomainCombiner();
                if (!(dc instanceof SubjectDomainCombiner)) {
                    return null;
                }
                SubjectDomainCombiner sdc = (SubjectDomainCombiner)dc;
                return sdc.getSubject();
            }
        });
    }
    public static <T> T doAs(final Subject subject, final java.security.PrivilegedAction<T> action) {
        java.lang.SecurityManager sm = System.getSecurityManager();
        if (sm != null) {
            sm.checkPermission(AuthPermissionHolder.DO_AS_PERMISSION);
        }
        Objects.requireNonNull(action, ResourcesMgr.getString("invalid.null.action.provided"));
        AccessControlContext currentAcc = AccessController.getContext();
        return java.security.AccessController.doPrivileged(action, createContext(subject, currentAcc));
    }
    public static <T> T doAs(final Subject subject, final java.security.PrivilegedExceptionAction<T> action) throws java.security.PrivilegedActionException {
        java.lang.SecurityManager sm = System.getSecurityManager();
        if (sm != null) {
            sm.checkPermission(AuthPermissionHolder.DO_AS_PERMISSION);
        }
        Objects.requireNonNull(action, ResourcesMgr.getString("invalid.null.action.provided"));
        AccessControlContext currentAcc = AccessController.getContext();
        return java.security.AccessController.doPrivileged(action, createContext(subject, currentAcc));
    }
    public static <T> T doAsPrivileged(final Subject subject, final java.security.PrivilegedAction<T> action, final java.security.AccessControlContext acc) {
        java.lang.SecurityManager sm = System.getSecurityManager();
        if (sm != null) {
            sm.checkPermission(AuthPermissionHolder.DO_AS_PRIVILEGED_PERMISSION);
        }
        Objects.requireNonNull(action, ResourcesMgr.getString("invalid.null.action.provided"));
        AccessControlContext callerAcc = acc == null ? new AccessControlContext(NULL_PD_ARRAY) : acc;
        return java.security.AccessController.doPrivileged(action, createContext(subject, callerAcc));
    }
    public static <T> T doAsPrivileged(final Subject subject, final java.security.PrivilegedExceptionAction<T> action, final java.security.AccessControlContext acc) throws java.security.PrivilegedActionException {
        java.lang.SecurityManager sm = System.getSecurityManager();
        if (sm != null) {
            sm.checkPermission(AuthPermissionHolder.DO_AS_PRIVILEGED_PERMISSION);
        }
        Objects.requireNonNull(action, ResourcesMgr.getString("invalid.null.action.provided"));
        AccessControlContext callerAcc = acc == null ? new AccessControlContext(NULL_PD_ARRAY) : acc;
        return java.security.AccessController.doPrivileged(action, createContext(subject, callerAcc));
    }
    private static AccessControlContext createContext(final Subject subject, final AccessControlContext acc) {
        return java.security.AccessController.doPrivileged(new java.security.PrivilegedAction<>() {
            public AccessControlContext run() {
                if (subject == null) {
                    return new AccessControlContext(acc, null);
                } else {
                    return new AccessControlContext
                                        (acc,
                                        new SubjectDomainCombiner(subject));
            }
            }
        });
    }
    public Set<Principal> getPrincipals() {
        return principals;
    }
    public <T extends Principal> Set<T> getPrincipals(Class<T> c) {
        Objects.requireNonNull(c, ResourcesMgr.getString("invalid.null.Class.provided"));
        return new ClassSet<T>(PRINCIPAL_SET, c);
    }
    public Set<Object> getPublicCredentials() {
        return pubCredentials;
    }
    public Set<Object> getPrivateCredentials() {
        return privCredentials;
    }
    public <T> Set<T> getPublicCredentials(Class<T> c) {
        Objects.requireNonNull(c, ResourcesMgr.getString("invalid.null.Class.provided"));
        return new ClassSet<T>(PUB_CREDENTIAL_SET, c);
    }
    public <T> Set<T> getPrivateCredentials(Class<T> c) {
        Objects.requireNonNull(c, ResourcesMgr.getString("invalid.null.Class.provided"));
        return new ClassSet<T>(PRIV_CREDENTIAL_SET, c);
    }
    @Override
    public boolean equals(Object o) {
        if (o == null) {
            return false;
        }
        if (this == o) {
            return true;
        }
        if (o instanceof Subject) {
            Subject that = (Subject) o;
            Set<Principal> thatPrincipals;
            synchronized (that.principals) {
                thatPrincipals = new HashSet<>(that.principals);
            }
            if (!principals.equals(thatPrincipals)) {
                return false;
            }
            Set<Object> thatPubCredentials;
            synchronized (that.pubCredentials) {
                thatPubCredentials = new HashSet<>(that.pubCredentials);
            }
            if (!pubCredentials.equals(thatPubCredentials)) {
                return false;
            }
            Set<Object> thatPrivCredentials;
            synchronized (that.privCredentials) {
                thatPrivCredentials = new HashSet<>(that.privCredentials);
            }
            return privCredentials.equals(thatPrivCredentials);
        }
        return false;
    }
    @Override
    public String toString() {
        return toString(true);
    }
    String toString(boolean includePrivateCredentials) {
        String s = ResourcesMgr.getString("Subject.");
        String suffix = "";
        synchronized (principals) {
            Iterator<Principal> pI = principals.iterator();
            while (pI.hasNext()) {
                Principal p = pI.next();
                suffix = suffix + ResourcesMgr.getString(".Principal.") + p.toString() + ResourcesMgr.getString("NEWLINE");
            }
        }
        synchronized (pubCredentials) {
            for (Object o : pubCredentials) {
                suffix = suffix + ResourcesMgr.getString(".Public.Credential.") + o.toString() + ResourcesMgr.getString("NEWLINE");
            }
        }
        if (includePrivateCredentials) {
            synchronized (privCredentials) {
                for (Object e : privCredentials) {
                    try {
                        suffix += ResourcesMgr.getString(".Private.Credential.") + e.toString() + ResourcesMgr.getString("NEWLINE");
                    } catch (SecurityException se) {
                        suffix += ResourcesMgr.getString(".Private.Credential.inaccessible.");
                        break;
                    }
                }
            }
        }
        return s + suffix;
    }
    @Override
    public int hashCode() {
        int hashCode = 0;
        synchronized (principals) {
            for (Object e : principals) {
                hashCode ^= e.hashCode();
            }
        }
        synchronized (pubCredentials) {
            for (Object e : pubCredentials) {
                hashCode ^= getCredHashCode(e);
            }
        }
        return hashCode;
    }
    private int getCredHashCode(Object o) {
        try {
            return o.hashCode();
        } catch (IllegalStateException ise) {
            return o.getClass().toString().hashCode();
        }
    }
    private void writeObject(java.io.ObjectOutputStream oos) throws java.io.IOException {
        synchronized (principals) {
            oos.defaultWriteObject();
        }
    }
    @SuppressWarnings("unchecked")
    private void readObject(java.io.ObjectInputStream s) throws java.io.IOException, ClassNotFoundException {
        ObjectInputStream.GetField gf = s.readFields();
        readOnly = gf.get("readOnly", false);
        Set<Principal> inputPrincs = (Set<Principal>) gf.get("principals", null);
        Objects.requireNonNull(inputPrincs, ResourcesMgr.getString("invalid.null.input.s."));
        try {
            principals = Collections.synchronizedSet(new SecureSet<>(this, PRINCIPAL_SET, inputPrincs));
        } catch (NullPointerException npe) {
            principals = Collections.synchronizedSet(new SecureSet<>(this, PRINCIPAL_SET));
        }
        this.pubCredentials = Collections.synchronizedSet(new SecureSet<>(this, PUB_CREDENTIAL_SET));
        this.privCredentials = Collections.synchronizedSet(new SecureSet<>(this, PRIV_CREDENTIAL_SET));
    }
    private static void collectionNullClean(Collection<?> coll) {
        boolean hasNullElements = false;
        Objects.requireNonNull(coll, ResourcesMgr.getString("invalid.null.input.s."));
        try {
            hasNullElements = coll.contains(null);
        } catch (NullPointerException npe) {
            hasNullElements = false;
        } finally {
            if (hasNullElements) {
                throw new NullPointerException(ResourcesMgr.getString("invalid.null.input.s."));
            }
        }
    }
    private static class SecureSet<E> implements Set<E>, java.io.Serializable {
        private static final long serialVersionUID = 7911754171111800359L;
        private static final ObjectStreamField[] serialPersistentFields = {new ObjectStreamField("this$0", Subject.class), new ObjectStreamField("elements", LinkedList.class), new ObjectStreamField("which", int.class)};
        Subject subject;
        LinkedList<E> elements;
        private int which;
        SecureSet(Subject subject, int which) {
            this.subject = subject;
            this.which = which;
            this.elements = new LinkedList<E>();
        }
        SecureSet(Subject subject, int which, Set<? extends E> set) {
            this.subject = subject;
            this.which = which;
            this.elements = new LinkedList<E>(set);
        }
        public int size() {
            return elements.size();
        }
        public Iterator<E> iterator() {
            LinkedList<E> list = elements;
            return new Iterator<E>() {
                ListIterator<E> i = list.listIterator(0);

                public boolean hasNext() {return i.hasNext();}

                public E next() {
                    if (which != Subject.PRIV_CREDENTIAL_SET) {
                        return i.next();
                    }

                    SecurityManager sm = System.getSecurityManager();
                    if (sm != null) {
                        try {
                            sm.checkPermission(new PrivateCredentialPermission
                                (list.get(i.nextIndex()).getClass().getName(),
                                subject.getPrincipals()));
                        } catch (SecurityException se) {
                            i.next();
                            throw (se);
                        }
                    }
                    return i.next();
                }

                public void remove() {

                    if (subject.isReadOnly()) {
                        throw new IllegalStateException(ResourcesMgr.getString
                                ("Subject.is.read.only"));
                    }

                    java.lang.SecurityManager sm = System.getSecurityManager();
                    if (sm != null) {
                        switch (which) {
                        case Subject.PRINCIPAL_SET:
                            sm.checkPermission(AuthPermissionHolder.MODIFY_PRINCIPALS_PERMISSION);
                            break;
                        case Subject.PUB_CREDENTIAL_SET:
                            sm.checkPermission(AuthPermissionHolder.MODIFY_PUBLIC_CREDENTIALS_PERMISSION);
                            break;
                        default:
                            sm.checkPermission(AuthPermissionHolder.MODIFY_PRIVATE_CREDENTIALS_PERMISSION);
                            break;
                        }
                    }
                    i.remove();
                }
            };
        }
        public boolean add(E o) {
            Objects.requireNonNull(o, ResourcesMgr.getString("invalid.null.input.s."));
            if (subject.isReadOnly()) {
                throw new IllegalStateException(ResourcesMgr.getString("Subject.is.read.only"));
            }
            java.lang.SecurityManager sm = System.getSecurityManager();
            if (sm != null) {
                switch (which) {
                    case Subject.PRINCIPAL_SET:
                        sm.checkPermission(AuthPermissionHolder.MODIFY_PRINCIPALS_PERMISSION);
                        break;
                    case Subject.PUB_CREDENTIAL_SET:
                        sm.checkPermission(AuthPermissionHolder.MODIFY_PUBLIC_CREDENTIALS_PERMISSION);
                        break;
                    default:
                        sm.checkPermission(AuthPermissionHolder.MODIFY_PRIVATE_CREDENTIALS_PERMISSION);
                        break;
                }
            }
            switch (which) {
                case Subject.PRINCIPAL_SET:
                    if (!(o instanceof Principal)) {
                        throw new SecurityException(ResourcesMgr.getString("attempting.to.add.an.object.which.is.not.an.instance.of.java.security.Principal.to.a.Subject.s.Principal.Set"));
                    }
                    break;
                default:
                    break;
            }
            return !elements.contains(o) ? elements.add(o) : false;
        }
        public boolean remove(Object o) {
            Objects.requireNonNull(o, ResourcesMgr.getString("invalid.null.input.s."));
            Iterator<E> e = iterator();
            while (e.hasNext()) {
                E next = which != Subject.PRIV_CREDENTIAL_SET ? e.next() : java.security.AccessController.doPrivileged(new java.security.PrivilegedAction<E>() {
                        public E run() {
                            return e.next();
                        }
                    });
                if (next.equals(o)) {
                    e.remove();
                    return true;
                }
            }
            return false;
        }
        public boolean contains(Object o) {
            Objects.requireNonNull(o, ResourcesMgr.getString("invalid.null.input.s."));
            Iterator<E> e = iterator();
            while (e.hasNext()) {
                E next;
                if (which != Subject.PRIV_CREDENTIAL_SET) {
                    next = e.next();
                } else {
                    SecurityManager sm = System.getSecurityManager();
                    if (sm != null) {
                        sm.checkPermission(new PrivateCredentialPermission(o.getClass().getName(), subject.getPrincipals()));
                    }
                    next = java.security.AccessController.doPrivileged(new java.security.PrivilegedAction<E>() {
                        public E run() {
                            return e.next();
                        }
                    });
                }
                if (next.equals(o)) {
                    return true;
                }
            }
            return false;
        }
        public boolean addAll(Collection<? extends E> c) {
            boolean result = false;
            collectionNullClean(c);
            for (E item : c) {
                result |= this.add(item);
            }
            return result;
        }
        public boolean removeAll(Collection<?> c) {
            collectionNullClean(c);
            boolean modified = false;
            Iterator<E> e = iterator();
            while (e.hasNext()) {
                E next = which != Subject.PRIV_CREDENTIAL_SET ? e.next() : java.security.AccessController.doPrivileged(new java.security.PrivilegedAction<E>() {
                        public E run() {
                            return e.next();
                        }
                    });
                for (Object e2 : c) {
                    if (next.equals(e2)) {
                        e.remove();
                        modified = true;
                        break;
                    }
                }
            }
            return modified;
        }
        public boolean containsAll(Collection<?> c) {
            collectionNullClean(c);
            for (Object item : c) {
                if (this.contains(item) == false) {
                    return false;
                }
            }
            return true;
        }
        public boolean retainAll(Collection<?> c) {
            collectionNullClean(c);
            boolean modified = false;
            Iterator<E> e = iterator();
            while (e.hasNext()) {
                E next = which != Subject.PRIV_CREDENTIAL_SET ? e.next() : java.security.AccessController.doPrivileged(new java.security.PrivilegedAction<E>() {
                        public E run() {
                            return e.next();
                        }
                    });
                if (c.contains(next) == false) {
                    e.remove();
                    modified = true;
                }
            }
            return modified;
        }
        public void clear() {
            Iterator<E> e = iterator();
            while (e.hasNext()) {
                E next = which != Subject.PRIV_CREDENTIAL_SET ? e.next() : java.security.AccessController.doPrivileged(new java.security.PrivilegedAction<E>() {
                        public E run() {
                            return e.next();
                        }
                    });
                e.remove();
            }
        }
        public boolean isEmpty() {
            return elements.isEmpty();
        }
        public Object[] toArray() {
            Iterator<E> e = iterator();
            while (e.hasNext()) {
                e.next();
            }
            return elements.toArray();
        }
        public <T> T[] toArray(T[] a) {
            Iterator<E> e = iterator();
            while (e.hasNext()) {
                e.next();
            }
            return elements.toArray(a);
        }
        public boolean equals(Object o) {
            if (o == this) {
                return true;
            }
            if (!(o instanceof Set)) {
                return false;
            }
            Collection<?> c = (Collection<?>) o;
            if (c.size() != size()) {
                return false;
            }
            try {
                return containsAll(c);
            } catch (ClassCastException unused) {
                return false;
            } catch (NullPointerException unused) {
                return false;
            }
        }
        public int hashCode() {
            int h = 0;
            Iterator<E> i = iterator();
            while (i.hasNext()) {
                E obj = i.next();
                if (obj != null) {
                    h += obj.hashCode();
                }
            }
            return h;
        }
        private void writeObject(java.io.ObjectOutputStream oos) throws java.io.IOException {
            if (which == Subject.PRIV_CREDENTIAL_SET) {
                Iterator<E> i = iterator();
                while (i.hasNext()) {
                    i.next();
                }
            }
            ObjectOutputStream.PutField fields = oos.putFields();
            fields.put("this$0", subject);
            fields.put("elements", elements);
            fields.put("which", which);
            oos.writeFields();
        }
        @SuppressWarnings("unchecked")
        private void readObject(ObjectInputStream ois) throws IOException, ClassNotFoundException {
            ObjectInputStream.GetField fields = ois.readFields();
            subject = (Subject) fields.get("this$0", null);
            which = fields.get("which", 0);
            LinkedList<E> tmp = (LinkedList<E>) fields.get("elements", null);
            Subject.collectionNullClean(tmp);
            elements = tmp.getClass() != LinkedList.class ? new LinkedList<E>(tmp) : tmp;
        }
    }
    private class ClassSet<T> extends AbstractSet<T> {
        private int which;
        private Class<T> c;
        private Set<T> set;
        ClassSet(int which, Class<T> c) {
            this.which = which;
            this.c = c;
            set = new HashSet<T>();
            switch (which) {
                case Subject.PRINCIPAL_SET:
                    synchronized (principals) {
                        populateSet();
                    }
                    break;
                case Subject.PUB_CREDENTIAL_SET:
                    synchronized (pubCredentials) {
                        populateSet();
                    }
                    break;
                default:
                    synchronized (privCredentials) {
                        populateSet();
                    }
                    break;
            }
        }
        @SuppressWarnings("unchecked")     /*To suppress warning from line 1374*/
        private void populateSet() {
            Iterator<?> iterator;
            switch (which) {
                case Subject.PRINCIPAL_SET:
                    iterator = Subject.this.principals.iterator();
                    break;
                case Subject.PUB_CREDENTIAL_SET:
                    iterator = Subject.this.pubCredentials.iterator();
                    break;
                default:
                    iterator = Subject.this.privCredentials.iterator();
                    break;
            }
            while (iterator.hasNext()) {
                Object next = which == Subject.PRIV_CREDENTIAL_SET ? java.security.AccessController.doPrivileged(new java.security.PrivilegedAction<>() {
                        public Object run() {
                            return iterator.next();
                        }
                    }) : iterator.next();
                if (c.isAssignableFrom(next.getClass())) {
                    if (which != Subject.PRIV_CREDENTIAL_SET) {
                        set.add((T) next);
                    } else {
                        SecurityManager sm = System.getSecurityManager();
                        if (sm != null) {
                            sm.checkPermission(new PrivateCredentialPermission(next.getClass().getName(), Subject.this.getPrincipals()));
                        }
                        set.add((T) next);
                    }
                }
            }
        }
        @Override
        public int size() {
            return set.size();
        }
        @Override
        public Iterator<T> iterator() {
            return set.iterator();
        }
        @Override
        public boolean add(T o) {
            if (!c.isAssignableFrom(o.getClass())) {
                MessageFormat form = new MessageFormat(ResourcesMgr.getString("attempting.to.add.an.object.which.is.not.an.instance.of.class"));
                Object[] source = {c.toString()};
                throw new SecurityException(form.format(source));
            }
            return set.add(o);
        }
    }
    static final class AuthPermissionHolder {
        static final AuthPermission DO_AS_PERMISSION = new AuthPermission("doAs");
        static final AuthPermission DO_AS_PRIVILEGED_PERMISSION = new AuthPermission("doAsPrivileged");
        static final AuthPermission SET_READ_ONLY_PERMISSION = new AuthPermission("setReadOnly");
        static final AuthPermission GET_SUBJECT_PERMISSION = new AuthPermission("getSubject");
        static final AuthPermission MODIFY_PRINCIPALS_PERMISSION = new AuthPermission("modifyPrincipals");
        static final AuthPermission MODIFY_PUBLIC_CREDENTIALS_PERMISSION = new AuthPermission("modifyPublicCredentials");
        static final AuthPermission MODIFY_PRIVATE_CREDENTIALS_PERMISSION = new AuthPermission("modifyPrivateCredentials");
    }
}
