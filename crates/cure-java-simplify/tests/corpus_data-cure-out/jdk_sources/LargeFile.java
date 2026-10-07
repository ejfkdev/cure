package com.sun.tools.javac.code;

import java.lang.ref.SoftReference;
import java.util.HashSet;
import java.util.HashMap;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.WeakHashMap;
import java.util.function.BiPredicate;
import java.util.function.Function;
import java.util.function.Predicate;
import java.util.stream.Collector;
import javax.tools.JavaFileObject;
import com.sun.tools.javac.code.Attribute.RetentionPolicy;
import com.sun.tools.javac.code.Lint.LintCategory;
import com.sun.tools.javac.code.Source.Feature;
import com.sun.tools.javac.code.Type.UndetVar.InferenceBound;
import com.sun.tools.javac.code.TypeMetadata.Entry.Kind;
import com.sun.tools.javac.comp.AttrContext;
import com.sun.tools.javac.comp.Check;
import com.sun.tools.javac.comp.Enter;
import com.sun.tools.javac.comp.Env;
import com.sun.tools.javac.comp.LambdaToMethod;
import com.sun.tools.javac.jvm.ClassFile;
import com.sun.tools.javac.util.*;
import static com.sun.tools.javac.code.BoundKind.*;
import static com.sun.tools.javac.code.Flags.*;
import static com.sun.tools.javac.code.Kinds.Kind.*;
import static com.sun.tools.javac.code.Scope.*;
import static com.sun.tools.javac.code.Scope.LookupKind.NON_RECURSIVE;
import static com.sun.tools.javac.code.Symbol.*;
import static com.sun.tools.javac.code.Type.*;
import static com.sun.tools.javac.code.TypeTag.*;
import static com.sun.tools.javac.jvm.ClassFile.externalize;
import com.sun.tools.javac.resources.CompilerProperties.Fragments;

public class Types {
    protected static final Context.Key<Types> typesKey = new Context.Key<>();
    final Symtab syms;
    final JavacMessages messages;
    final Names names;
    final boolean allowDefaultMethods;
    final boolean mapCapturesToBounds;
    final Check chk;
    final Enter enter;
    JCDiagnostic.Factory diags;
    List<Warner> warnStack = List.nil();
    final Name capturedName;
    public final Warner noWarnings;
    public static Types instance(Context context) {
        Types instance = context.get(typesKey);
        if (instance == null) 
            instance = new Types(context);
        return instance;
    }
    protected Types(Context context) {
        context.put(typesKey, this);
        syms = Symtab.instance(context);
        names = Names.instance(context);
        Source source = Source.instance(context);
        allowDefaultMethods = Feature.DEFAULT_METHODS.allowedInSource(source);
        mapCapturesToBounds = Feature.MAP_CAPTURES_TO_BOUNDS.allowedInSource(source);
        chk = Check.instance(context);
        enter = Enter.instance(context);
        capturedName = names.fromString("<captured wildcard>");
        messages = JavacMessages.instance(context);
        diags = JCDiagnostic.Factory.instance(context);
        noWarnings = new Warner(null);
    }
    public Type wildUpperBound(Type t) {
        if (t.hasTag(WILDCARD)) {
            WildcardType w = (WildcardType) t;
            return w.isSuperBound() ? w.bound == null ? syms.objectType : w.bound.getUpperBound() : wildUpperBound(w.type);
        } else 
            return t;
    }
    public Type cvarUpperBound(Type t) {
        if (t.hasTag(TYPEVAR)) {
            TypeVar v = (TypeVar) t;
            return v.isCaptured() ? cvarUpperBound(v.getUpperBound()) : v;
        } else 
            return t;
    }
    public Type wildLowerBound(Type t) {
        if (t.hasTag(WILDCARD)) {
            WildcardType w = (WildcardType) t;
            return w.isExtendsBound() ? syms.botType : wildLowerBound(w.type);
        } else 
            return t;
    }
    public Type cvarLowerBound(Type t) {
        return t.hasTag(TYPEVAR) && ((TypeVar) t).isCaptured() ? cvarLowerBound(t.getLowerBound()) : t;
    }
    public Type skipTypeVars(Type site, boolean capture) {
        while (site.hasTag(TYPEVAR)) {
            site = site.getUpperBound();
        }
        return capture ? capture(site) : site;
    }
    enum ProjectionKind {
        UPWARDS() {
            @Override
            ProjectionKind complement() {
                return DOWNWARDS;
            }
        }, DOWNWARDS() {
            @Override
            ProjectionKind complement() {
                return UPWARDS;
            }
        };
        abstract ProjectionKind complement();
    }
    class TypeProjection extends TypeMapping<ProjectionKind> {
        List<Type> vars;
        Set<Type> seen = new HashSet<>();
        public TypeProjection(List<Type> vars) {
            this.vars = vars;
        }
        @Override
        public Type visitClassType(ClassType t, ProjectionKind pkind) {
            if (t.isCompound()) {
                List<Type> components = directSupertypes(t);
                List<Type> components1 = components.map((c) -> c.map(this, pkind));
                return components == components1 ? t : makeIntersectionType(components1);
            } else {
                Type outer = t.getEnclosingType();
                Type outer1 = visit(outer, pkind);
                List<Type> typarams = t.getTypeArguments();
                List<Type> formals = t.tsym.type.getTypeArguments();
                ListBuffer<Type> typarams1 = new ListBuffer<>();
                boolean changed = false;
                for (Type actual : typarams) {
                    Type t2 = mapTypeArgument(t, formals.head.getUpperBound(), actual, pkind);
                    if (t2.hasTag(BOT)) {
                        return syms.botType;
                    }
                    typarams1.add(t2);
                    changed |= actual != t2;
                    formals = formals.tail;
                }
                return outer1 == outer && !changed ? t : new ClassType(outer1, typarams1.toList(), t.tsym, t.getMetadata()) {
                    @Override
                    protected boolean needsStripping() {
                        return true;
                    }
                };
            }
        }
        @Override
        public Type visitArrayType(ArrayType t, ProjectionKind s) {
            Type elemtype = t.elemtype;
            Type elemtype1 = visit(elemtype, s);
            return elemtype1 == elemtype ? t : elemtype1.hasTag(BOT) ? syms.botType : new ArrayType(elemtype1, t.tsym, t.metadata) {
                    @Override
                    protected boolean needsStripping() {
                        return true;
                    }
                };
        }
        @Override
        public Type visitTypeVar(TypeVar t, ProjectionKind pkind) {
            if (vars.contains(t)) {
                if (seen.add(t)) {
                    try {
                        Type bound;
                        switch (pkind) {
                            case UPWARDS:
                                bound = t.getUpperBound();
                                break;
                            case DOWNWARDS:
                                bound = t.getLowerBound() == null ? syms.botType : t.getLowerBound();
                                break;
                            default:
                                Assert.error();
                                return null;
                        }
                        return bound.map(this, pkind);
                    } finally {
                        seen.remove(t);
                    }
                } else {
                    return pkind == ProjectionKind.UPWARDS ? syms.objectType : syms.botType;
                }
            } else {
                return t;
            }
        }
        private Type mapTypeArgument(Type site, Type declaredBound, Type t, ProjectionKind pkind) {
            return t.containsAny(vars) ? t.map(new TypeArgumentProjection(site, declaredBound), pkind) : t;
        }
        class TypeArgumentProjection extends TypeMapping<ProjectionKind> {
            Type site;
            Type declaredBound;
            TypeArgumentProjection(Type site, Type declaredBound) {
                this.site = site;
                this.declaredBound = declaredBound;
            }
            @Override
            public Type visitType(Type t, ProjectionKind pkind) {
                if (pkind == ProjectionKind.DOWNWARDS) {
                    return syms.botType;
                }
                Type upper = t.map(TypeProjection.this, ProjectionKind.UPWARDS);
                Type lower = t.map(TypeProjection.this, ProjectionKind.DOWNWARDS);
                List<Type> formals = site.tsym.type.getTypeArguments();
                BoundKind bk;
                Type bound;
                if (!isSameType(upper, syms.objectType) && (declaredBound.containsAny(formals) || !isSubtype(declaredBound, upper))) {
                    bound = upper;
                    bk = EXTENDS;
                } else if (!lower.hasTag(BOT)) {
                    bound = lower;
                    bk = SUPER;
                } else {
                    bound = syms.objectType;
                    bk = UNBOUND;
                }
                return makeWildcard(bound, bk);
            }
            @Override
            public Type visitWildcardType(WildcardType wt, ProjectionKind pkind) {
                Type bound = syms.botType;
                BoundKind bk = wt.kind;
                switch (wt.kind) {
                    case EXTENDS:
                        bound = wt.type.map(TypeProjection.this, pkind);
                        if (bound.hasTag(BOT)) {
                            return syms.botType;
                        }
                        break;
                    case SUPER:
                        bound = wt.type.map(TypeProjection.this, pkind.complement());
                        if (bound.hasTag(BOT)) {
                            bound = syms.objectType;
                            bk = UNBOUND;
                        }
                        break;
                }
                return makeWildcard(bound, bk);
            }
            private Type makeWildcard(Type bound, BoundKind bk) {
                return new WildcardType(bound, bk, syms.boundClass) {
                    @Override
                    protected boolean needsStripping() {
                        return true;
                    }
                };
            }
        }
    }
    public Type upward(Type t, List<Type> vars) {
        return t.map(new TypeProjection(vars), ProjectionKind.UPWARDS);
    }
    public List<Type> captures(Type t) {
        CaptureScanner cs = new CaptureScanner();
        Set<Type> captures = new HashSet<>();
        cs.visit(t, captures);
        return List.from(captures);
    }
    class CaptureScanner extends SimpleVisitor<Void, Set<Type>> {
        @Override
        public Void visitType(Type t, Set<Type> types) {
            return null;
        }
        @Override
        public Void visitClassType(ClassType t, Set<Type> seen) {
            if (t.isCompound()) {
                directSupertypes(t).forEach((s) -> visit(s, seen));
            } else {
                t.allparams().forEach((ta) -> visit(ta, seen));
            }
            return null;
        }
        @Override
        public Void visitArrayType(ArrayType t, Set<Type> seen) {
            return visit(t.elemtype, seen);
        }
        @Override
        public Void visitWildcardType(WildcardType t, Set<Type> seen) {
            visit(t.type, seen);
            return null;
        }
        @Override
        public Void visitTypeVar(TypeVar t, Set<Type> seen) {
            if ((t.tsym.flags() & Flags.SYNTHETIC) != 0 && seen.add(t)) {
                visit(t.getUpperBound(), seen);
            }
            return null;
        }
        @Override
        public Void visitCapturedType(CapturedType t, Set<Type> seen) {
            if (seen.add(t)) {
                visit(t.getUpperBound(), seen);
                visit(t.getLowerBound(), seen);
            }
            return null;
        }
    }
    public boolean isUnbounded(Type t) {
        return isUnbounded.visit(t);
    }
    private final UnaryVisitor<Boolean> isUnbounded = new UnaryVisitor<Boolean>() {

            public Boolean visitType(Type t, Void ignored) {
                return true;
            }

            @Override
            public Boolean visitClassType(ClassType t, Void ignored) {
                List<Type> parms = t.tsym.type.allparams();
                List<Type> args = t.allparams();
                while (parms.nonEmpty()) {
                    WildcardType unb = new WildcardType(syms.objectType,
                                                        BoundKind.UNBOUND,
                                                        syms.boundClass,
                                                        (TypeVar)parms.head);
                    if (!containsType(args.head, unb))
                        return false;
                    parms = parms.tail;
                    args = args.tail;
                }
                return true;
            }
        };
    public Type asSub(Type t, Symbol sym) {
        return asSub.visit(t, sym);
    }
    private final SimpleVisitor<Type,Symbol> asSub = new SimpleVisitor<Type,Symbol>() {

            public Type visitType(Type t, Symbol sym) {
                return null;
            }

            @Override
            public Type visitClassType(ClassType t, Symbol sym) {
                if (t.tsym == sym)
                    return t;
                Type base = asSuper(sym.type, t.tsym);
                if (base == null)
                    return null;
                ListBuffer<Type> from = new ListBuffer<>();
                ListBuffer<Type> to = new ListBuffer<>();
                try {
                    adapt(base, t, from, to);
                } catch (AdaptFailure ex) {
                    return null;
                }
                Type res = subst(sym.type, from.toList(), to.toList());
                if (!isSubtype(res, t))
                    return null;
                ListBuffer<Type> openVars = new ListBuffer<>();
                for (List<Type> l = sym.type.allparams();
                     l.nonEmpty(); l = l.tail)
                    if (res.contains(l.head) && !t.contains(l.head))
                        openVars.append(l.head);
                if (openVars.nonEmpty()) {
                    if (t.isRaw()) {
                        // The subtype of a raw type is raw
                        res = erasure(res);
                    } else {
                        // Unbound type arguments default to ?
                        List<Type> opens = openVars.toList();
                        ListBuffer<Type> qs = new ListBuffer<>();
                        for (List<Type> iter = opens; iter.nonEmpty(); iter = iter.tail) {
                            qs.append(new WildcardType(syms.objectType, BoundKind.UNBOUND,
                                                       syms.boundClass, (TypeVar) iter.head));
                        }
                        res = subst(res, opens, qs.toList());
                    }
                }
                return res;
            }

            @Override
            public Type visitErrorType(ErrorType t, Symbol sym) {
                return t;
            }
        };
    public boolean isConvertible(Type t, Type s, Warner warn) {
        if (t.hasTag(ERROR)) {
            return true;
        }
        boolean tPrimitive = t.isPrimitive();
        boolean sPrimitive = s.isPrimitive();
        if (tPrimitive == sPrimitive) {
            return isSubtypeUnchecked(t, s, warn);
        }
        boolean tUndet = t.hasTag(UNDETVAR);
        boolean sUndet = s.hasTag(UNDETVAR);
        return tUndet || sUndet ? tUndet ? isSubtype(t, boxedTypeOrType(s)) : isSubtype(boxedTypeOrType(t), s) : tPrimitive ? isSubtype(boxedClass(t).type, s) : isSubtype(unboxedType(t), s);
    }
    public boolean isConvertible(Type t, Type s) {
        return isConvertible(t, s, noWarnings);
    }
    public static class FunctionDescriptorLookupError extends RuntimeException {
        private static final long serialVersionUID = 0;
        transient JCDiagnostic diagnostic;
        FunctionDescriptorLookupError() {
            this.diagnostic = null;
        }
        FunctionDescriptorLookupError setMessage(JCDiagnostic diag) {
            this.diagnostic = diag;
            return this;
        }
        public JCDiagnostic getDiagnostic() {
            return diagnostic;
        }
        @Override
        public Throwable fillInStackTrace() {
            return this;
        }
    }
    class DescriptorCache {
        private WeakHashMap<TypeSymbol, Entry> _map = new WeakHashMap<>();
        class FunctionDescriptor {
            Symbol descSym;
            FunctionDescriptor(Symbol descSym) {
                this.descSym = descSym;
            }
            public Symbol getSymbol() {
                return descSym;
            }
            public Type getType(Type site) {
                site = removeWildcards(site);
                if (site.isIntersection()) {
                    IntersectionClassType ict = (IntersectionClassType) site;
                    for (Type component : ict.getExplicitComponents()) {
                        if (!chk.checkValidGenericType(component)) {
                            throw failure(diags.fragment(Fragments.NoSuitableFunctionalIntfInst(site)));
                        }
                    }
                } else {
                    if (!chk.checkValidGenericType(site)) {
                        throw failure(diags.fragment(Fragments.NoSuitableFunctionalIntfInst(site)));
                    }
                }
                return memberType(site, descSym);
            }
        }
        class Entry {
            final FunctionDescriptor cachedDescRes;
            final int prevMark;
            public Entry(FunctionDescriptor cachedDescRes, int prevMark) {
                this.cachedDescRes = cachedDescRes;
                this.prevMark = prevMark;
            }
            boolean matches(int mark) {
                return this.prevMark == mark;
            }
        }
        FunctionDescriptor get(TypeSymbol origin) throws FunctionDescriptorLookupError {
            Entry e = _map.get(origin);
            CompoundScope members = membersClosure(origin.type, false);
            if (e == null || !e.matches(members.getMark())) {
                FunctionDescriptor descRes = findDescriptorInternal(origin, members);
                _map.put(origin, new Entry(descRes, members.getMark()));
                return descRes;
            } else {
                return e.cachedDescRes;
            }
        }
        public FunctionDescriptor findDescriptorInternal(TypeSymbol origin, CompoundScope membersCache) throws FunctionDescriptorLookupError {
            if (!origin.isInterface() || (origin.flags() & ANNOTATION) != 0 || origin.isSealed()) {
                throw failure("not.a.functional.intf", origin);
            }
            ListBuffer<Symbol> abstracts = new ListBuffer<>();
            for (Symbol sym : membersCache.getSymbols(new DescriptorFilter(origin))) {
                Type mtype = memberType(origin.type, sym);
                if (abstracts.isEmpty()) {
                    abstracts.append(sym);
                } else if (sym.name == abstracts.first().name && overrideEquivalent(mtype, memberType(origin.type, abstracts.first()))) {
                    if (!abstracts.stream().filter((msym) -> msym.owner.isSubClass(sym.enclClass(), Types.this)).map((msym) -> memberType(origin.type, msym)).anyMatch((abstractMType) -> isSubSignature(abstractMType, mtype))) {
                        abstracts.append(sym);
                    }
                } else {
                    throw failure("not.a.functional.intf.1", origin, diags.fragment(Fragments.IncompatibleAbstracts(Kinds.kindName(origin), origin)));
                }
            }
            if (abstracts.isEmpty()) {
                throw failure("not.a.functional.intf.1", origin, diags.fragment(Fragments.NoAbstracts(Kinds.kindName(origin), origin)));
            } else if (abstracts.size() == 1) {
                return new FunctionDescriptor(abstracts.first());
            } else {
                FunctionDescriptor descRes = mergeDescriptors(origin, abstracts.toList());
                if (descRes == null) {
                    ListBuffer<JCDiagnostic> descriptors = new ListBuffer<>();
                    for (Symbol desc : abstracts) {
                        String key = desc.type.getThrownTypes().nonEmpty() ? "descriptor.throws" : "descriptor";
                        descriptors.append(diags.fragment(key, desc.name, desc.type.getParameterTypes(), desc.type.getReturnType(), desc.type.getThrownTypes()));
                    }
                    throw failure(new JCDiagnostic.MultilineDiagnostic(diags.fragment(Fragments.IncompatibleDescsInFunctionalIntf(Kinds.kindName(origin), origin)), descriptors.toList()));
                }
                return descRes;
            }
        }
        private FunctionDescriptor mergeDescriptors(TypeSymbol origin, List<Symbol> methodSyms) {
            return mergeAbstracts(methodSyms, origin.type, false).map((bestSoFar) -> new FunctionDescriptor(bestSoFar.baseSymbol()) {
                        @Override
                        public Type getType(Type origin) {
                            Type mt = memberType(origin, getSymbol());
                            return createMethodTypeWithThrown(mt, bestSoFar.type.getThrownTypes());
                        }
                    }).orElse(null);
        }
        FunctionDescriptorLookupError failure(String msg, Object... args) {
            return failure(diags.fragment(msg, args));
        }
        FunctionDescriptorLookupError failure(JCDiagnostic diag) {
            return new FunctionDescriptorLookupError().setMessage(diag);
        }
    }
    private DescriptorCache descCache = new DescriptorCache();
    public Symbol findDescriptorSymbol(TypeSymbol origin) throws FunctionDescriptorLookupError {
        return descCache.get(origin).getSymbol();
    }
    public Type findDescriptorType(Type origin) throws FunctionDescriptorLookupError {
        return descCache.get(origin.tsym).getType(origin);
    }
    public boolean isFunctionalInterface(TypeSymbol tsym) {
        try {
            findDescriptorSymbol(tsym);
            return true;
        } catch (FunctionDescriptorLookupError ex) {
            return false;
        }
    }
    public boolean isFunctionalInterface(Type site) {
        try {
            findDescriptorType(site);
            return true;
        } catch (FunctionDescriptorLookupError ex) {
            return false;
        }
    }
    public Type removeWildcards(Type site) {
        if (site.getTypeArguments().stream().anyMatch((t) -> t.hasTag(WILDCARD))) {
            List<Type> actuals = site.getTypeArguments();
            List<Type> formals = site.tsym.type.getTypeArguments();
            ListBuffer<Type> targs = new ListBuffer<>();
            for (Type formal : formals) {
                Type actual = actuals.head;
                Type bound = formal.getUpperBound();
                if (actuals.head.hasTag(WILDCARD)) {
                    WildcardType wt = (WildcardType) actual;
                    if (bound.containsAny(formals)) {
                        targs.add(wt.type);
                    } else {
                        switch (wt.kind) {
                            case UNBOUND:
                                targs.add(bound);
                                break;
                            case EXTENDS:
                                targs.add(glb(bound, wt.type));
                                break;
                            case SUPER:
                                targs.add(wt.type);
                                break;
                            default:
                                Assert.error("Cannot get here!");
                        }
                    }
                } else {
                    targs.add(actual);
                }
                actuals = actuals.tail;
            }
            return subst(site.tsym.type, formals, targs.toList());
        } else {
            return site;
        }
    }
    public ClassSymbol makeFunctionalInterfaceClass(Env<AttrContext> env, Name name, Type target, long cflags) {
        if (target == null || target == syms.unknownType) {
            return null;
        }
        Symbol descSym = findDescriptorSymbol(target.tsym);
        Type descType = findDescriptorType(target);
        ClassSymbol csym = new ClassSymbol(cflags, name, env.enclClass.sym.outermostClass());
        csym.completer = Completer.NULL_COMPLETER;
        csym.members_field = WriteableScope.create(csym);
        MethodSymbol instDescSym = new MethodSymbol(descSym.flags(), descSym.name, descType, csym);
        csym.members_field.enter(instDescSym);
        Type.ClassType ctype = new Type.ClassType(Type.noType, List.nil(), csym);
        ctype.supertype_field = syms.objectType;
        ctype.interfaces_field = target.isIntersection() ? directSupertypes(target) : List.of(target);
        csym.type = ctype;
        csym.sourcefile = ((ClassSymbol) csym.owner).sourcefile;
        return csym;
    }
    public List<Symbol> functionalInterfaceBridges(TypeSymbol origin) {
        Assert.check(isFunctionalInterface(origin));
        Symbol descSym = findDescriptorSymbol(origin);
        CompoundScope members = membersClosure(origin.type, false);
        ListBuffer<Symbol> overridden = new ListBuffer<>();
        outer:
            for (Symbol m2 : members.getSymbolsByName(descSym.name, bridgeFilter)) {
                if (m2 == descSym) 
                    continue; else if (descSym.overrides(m2, origin, Types.this, false)) {
                    for (Symbol m3 : overridden) {
                        if (isSameType(m3.erasure(Types.this), m2.erasure(Types.this)) || m3.overrides(m2, origin, Types.this, false) && (pendingBridges((ClassSymbol) origin, m3.enclClass()) || ((MethodSymbol) m2).binaryImplementation((ClassSymbol) m3.owner, Types.this) != null)) {
                            continue outer;
                        }
                    }
                    overridden.add(m2);
                }
            }
        return overridden.toList();
    }
    private Predicate<Symbol> bridgeFilter = new Predicate<Symbol>() {
            public boolean test(Symbol t) {
                return t.kind == MTH &&
                        t.name != names.init &&
                        t.name != names.clinit &&
                        (t.flags() & SYNTHETIC) == 0;
            }
        };
    private boolean pendingBridges(ClassSymbol origin, TypeSymbol s) {
        if (origin.classfile != null && origin.classfile.getKind() == JavaFileObject.Kind.CLASS && enter.getEnv(origin) == null) {
            return false;
        }
        if (origin == s) {
            return true;
        }
        for (Type t : interfaces(origin.type)) {
            if (pendingBridges((ClassSymbol) t.tsym, s)) {
                return true;
            }
        }
        return false;
    }
    class DescriptorFilter implements Predicate<Symbol> {
        TypeSymbol origin;
        DescriptorFilter(TypeSymbol origin) {
            this.origin = origin;
        }
        @Override
       public boolean test(Symbol sym) {
            return sym.kind == MTH && (sym.flags() & (ABSTRACT | DEFAULT)) == ABSTRACT && !overridesObjectMethod(origin, sym) && (interfaceCandidates(origin.type, (MethodSymbol) sym).head.flags() & DEFAULT) == 0;
        }
    }
    public boolean isSubtypeUnchecked(Type t, Type s) {
        return isSubtypeUnchecked(t, s, noWarnings);
    }
    public boolean isSubtypeUnchecked(Type t, Type s, Warner warn) {
        boolean result = isSubtypeUncheckedInternal(t, s, true, warn);
        if (result) {
            checkUnsafeVarargsConversion(t, s, warn);
        }
        return result;
    }
    private boolean isSubtypeUncheckedInternal(Type t, Type s, boolean capture, Warner warn) {
        if (t.hasTag(ARRAY) && s.hasTag(ARRAY)) {
            return ((ArrayType) t).elemtype.isPrimitive() ? isSameType(elemtype(t), elemtype(s)) : isSubtypeUncheckedInternal(elemtype(t), elemtype(s), false, warn);
        } else if (isSubtype(t, s, capture)) {
            return true;
        } else if (t.hasTag(TYPEVAR)) {
            return isSubtypeUncheckedInternal(t.getUpperBound(), s, false, warn);
        } else if (!s.isRaw()) {
            Type t2 = asSuper(t, s.tsym);
            if (t2 != null && t2.isRaw()) {
                if (isReifiable(s)) {
                    warn.silentWarn(LintCategory.UNCHECKED);
                } else {
                    warn.warn(LintCategory.UNCHECKED);
                }
                return true;
            }
        }
        return false;
    }
    private void checkUnsafeVarargsConversion(Type t, Type s, Warner warn) {
        if (!t.hasTag(ARRAY) || isReifiable(t)) {
            return;
        }
        ArrayType from = (ArrayType) t;
        boolean shouldWarn = false;
        switch (s.getTag()) {
            case ARRAY:
                ArrayType to = (ArrayType) s;
                shouldWarn = from.isVarargs() && !to.isVarargs() && !isReifiable(from);
                break;
            case CLASS:
                shouldWarn = from.isVarargs();
                break;
        }
        if (shouldWarn) {
            warn.warn(LintCategory.VARARGS);
        }
    }
    public final boolean isSubtype(Type t, Type s) {
        return isSubtype(t, s, true);
    }
    public final boolean isSubtypeNoCapture(Type t, Type s) {
        return isSubtype(t, s, false);
    }
    public boolean isSubtype(Type t, Type s, boolean capture) {
        if (t.equalsIgnoreMetadata(s)) 
            return true;
        if (s.isPartial()) 
            return isSuperType(s, t);
        if (s.isCompound()) {
            for (Type s2 : interfaces(s).prepend(supertype(s))) {
                if (!isSubtype(t, s2, capture)) 
                    return false;
            }
            return true;
        }
        if (!t.hasTag(UNDETVAR) && !t.isCompound()) {
            Type lower = cvarLowerBound(wildLowerBound(s));
            if (s != lower && !lower.hasTag(BOT)) 
                return isSubtype(capture ? capture(t) : t, lower, false);
        }
        return isSubtype.visit(capture ? capture(t) : t, s);
    }
    private TypeRelation isSubtype = new TypeRelation() {
            @Override
            public Boolean visitType(Type t, Type s) {
                switch (t.getTag()) {
                 case BYTE:
                     return (!s.hasTag(CHAR) && t.getTag().isSubRangeOf(s.getTag()));
                 case CHAR:
                     return (!s.hasTag(SHORT) && t.getTag().isSubRangeOf(s.getTag()));
                 case SHORT: case INT: case LONG:
                 case FLOAT: case DOUBLE:
                     return t.getTag().isSubRangeOf(s.getTag());
                 case BOOLEAN: case VOID:
                     return t.hasTag(s.getTag());
                 case TYPEVAR:
                     return isSubtypeNoCapture(t.getUpperBound(), s);
                 case BOT:
                     return
                         s.hasTag(BOT) || s.hasTag(CLASS) ||
                         s.hasTag(ARRAY) || s.hasTag(TYPEVAR);
                 case WILDCARD: //we shouldn't be here - avoids crash (see 7034495)
                 case NONE:
                     return false;
                 default:
                     throw new AssertionError("isSubtype " + t.getTag());
                 }
            }

            private Set<TypePair> cache = new HashSet<>();

            private boolean containsTypeRecursive(Type t, Type s) {
                TypePair pair = new TypePair(t, s);
                if (cache.add(pair)) {
                    try {
                        return containsType(t.getTypeArguments(),
                                            s.getTypeArguments());
                    } finally {
                        cache.remove(pair);
                    }
                } else {
                    return containsType(t.getTypeArguments(),
                                        rewriteSupers(s).getTypeArguments());
                }
            }

            private Type rewriteSupers(Type t) {
                if (!t.isParameterized())
                    return t;
                ListBuffer<Type> from = new ListBuffer<>();
                ListBuffer<Type> to = new ListBuffer<>();
                adaptSelf(t, from, to);
                if (from.isEmpty())
                    return t;
                ListBuffer<Type> rewrite = new ListBuffer<>();
                boolean changed = false;
                for (Type orig : to.toList()) {
                    Type s = rewriteSupers(orig);
                    if (s.isSuperBound() && !s.isExtendsBound()) {
                        s = new WildcardType(syms.objectType,
                                             BoundKind.UNBOUND,
                                             syms.boundClass,
                                             s.getMetadata());
                        changed = true;
                    } else if (s != orig) {
                        s = new WildcardType(wildUpperBound(s),
                                             BoundKind.EXTENDS,
                                             syms.boundClass,
                                             s.getMetadata());
                        changed = true;
                    }
                    rewrite.append(s);
                }
                if (changed)
                    return subst(t.tsym.type, from.toList(), rewrite.toList());
                else
                    return t;
            }

            @Override
            public Boolean visitClassType(ClassType t, Type s) {
                Type sup = asSuper(t, s.tsym);
                if (sup == null) return false;
                // If t is an intersection, sup might not be a class type
                if (!sup.hasTag(CLASS)) return isSubtypeNoCapture(sup, s);
                return sup.tsym == s.tsym
                     // Check type variable containment
                    && (!s.isParameterized() || containsTypeRecursive(s, sup))
                    && isSubtypeNoCapture(sup.getEnclosingType(),
                                          s.getEnclosingType());
            }

            @Override
            public Boolean visitArrayType(ArrayType t, Type s) {
                if (s.hasTag(ARRAY)) {
                    if (t.elemtype.isPrimitive())
                        return isSameType(t.elemtype, elemtype(s));
                    else
                        return isSubtypeNoCapture(t.elemtype, elemtype(s));
                }

                if (s.hasTag(CLASS)) {
                    Name sname = s.tsym.getQualifiedName();
                    return sname == names.java_lang_Object
                        || sname == names.java_lang_Cloneable
                        || sname == names.java_io_Serializable;
                }

                return false;
            }

            @Override
            public Boolean visitUndetVar(UndetVar t, Type s) {
                //todo: test against origin needed? or replace with substitution?
                if (t == s || t.qtype == s || s.hasTag(ERROR) || s.hasTag(UNKNOWN)) {
                    return true;
                } else if (s.hasTag(BOT)) {
                    //if 's' is 'null' there's no instantiated type U for which
                    //U <: s (but 'null' itself, which is not a valid type)
                    return false;
                }

                t.addBound(InferenceBound.UPPER, s, Types.this);
                return true;
            }

            @Override
            public Boolean visitErrorType(ErrorType t, Type s) {
                return true;
            }
        };
    public boolean isSubtypeUnchecked(Type t, List<Type> ts, Warner warn) {
        for (List<Type> l = ts; l.nonEmpty(); l = l.tail) 
            if (!isSubtypeUnchecked(t, l.head, warn)) 
                return false;
        return true;
    }
    public boolean isSubtypes(List<Type> ts, List<Type> ss) {
        while (ts.tail != null && ss.tail != null && isSubtype(ts.head, ss.head)) {
            ts = ts.tail;
            ss = ss.tail;
        }
        return ts.tail == null && ss.tail == null;
    }
    public boolean isSubtypesUnchecked(List<Type> ts, List<Type> ss, Warner warn) {
        while (ts.tail != null && ss.tail != null && isSubtypeUnchecked(ts.head, ss.head, warn)) {
            ts = ts.tail;
            ss = ss.tail;
        }
        return ts.tail == null && ss.tail == null;
    }
    public boolean isSuperType(Type t, Type s) {
        switch (t.getTag()) {
            case ERROR:
                return true;
            case UNDETVAR:
                {
                    UndetVar undet = (UndetVar) t;
                    if (t == s || undet.qtype == s || s.hasTag(ERROR) || s.hasTag(BOT)) {
                        return true;
                    }
                    undet.addBound(InferenceBound.LOWER, s, this);
                    return true;
                }
            default:
                return isSubtype(s, t);
        }
    }
    public boolean isSameTypes(List<Type> ts, List<Type> ss) {
        while (ts.tail != null && ss.tail != null && isSameType(ts.head, ss.head)) {
            ts = ts.tail;
            ss = ss.tail;
        }
        return ts.tail == null && ss.tail == null;
    }
    public boolean isSignaturePolymorphic(MethodSymbol msym) {
        List<Type> argtypes = msym.type.getParameterTypes();
        return (msym.flags_field & NATIVE) != 0 && (msym.owner == syms.methodHandleType.tsym || msym.owner == syms.varHandleType.tsym) && argtypes.length() == 1 && argtypes.head.hasTag(TypeTag.ARRAY) && ((ArrayType) argtypes.head).elemtype.tsym == syms.objectType.tsym;
    }
    public boolean isSameType(Type t, Type s) {
        return isSameTypeVisitor.visit(t, s);
    }
    TypeRelation isSameTypeVisitor = new TypeRelation() {

            public Boolean visitType(Type t, Type s) {
                if (t.equalsIgnoreMetadata(s))
                    return true;

                if (s.isPartial())
                    return visit(s, t);

                switch (t.getTag()) {
                case BYTE: case CHAR: case SHORT: case INT: case LONG: case FLOAT:
                case DOUBLE: case BOOLEAN: case VOID: case BOT: case NONE:
                    return t.hasTag(s.getTag());
                case TYPEVAR: {
                    if (s.hasTag(TYPEVAR)) {
                        //type-substitution does not preserve type-var types
                        //check that type var symbols and bounds are indeed the same
                        return t == s;
                    }
                    else {
                        //special case for s == ? super X, where upper(s) = u
                        //check that u == t, where u has been set by Type.withTypeVar
                        return s.isSuperBound() &&
                                !s.isExtendsBound() &&
                                visit(t, wildUpperBound(s));
                    }
                }
                default:
                    throw new AssertionError("isSameType " + t.getTag());
                }
            }

            @Override
            public Boolean visitWildcardType(WildcardType t, Type s) {
                if (!s.hasTag(WILDCARD)) {
                    return false;
                } else {
                    WildcardType t2 = (WildcardType)s;
                    return (t.kind == t2.kind || (t.isExtendsBound() && s.isExtendsBound())) &&
                            isSameType(t.type, t2.type);
                }
            }

            @Override
            public Boolean visitClassType(ClassType t, Type s) {
                if (t == s)
                    return true;

                if (s.isPartial())
                    return visit(s, t);

                if (s.isSuperBound() && !s.isExtendsBound())
                    return visit(t, wildUpperBound(s)) && visit(t, wildLowerBound(s));

                if (t.isCompound() && s.isCompound()) {
                    if (!visit(supertype(t), supertype(s)))
                        return false;

                    Map<Symbol,Type> tMap = new HashMap<>();
                    for (Type ti : interfaces(t)) {
                        if (tMap.containsKey(ti)) {
                            throw new AssertionError("Malformed intersection");
                        }
                        tMap.put(ti.tsym, ti);
                    }
                    for (Type si : interfaces(s)) {
                        if (!tMap.containsKey(si.tsym))
                            return false;
                        Type ti = tMap.remove(si.tsym);
                        if (!visit(ti, si))
                            return false;
                    }
                    return tMap.isEmpty();
                }
                return t.tsym == s.tsym
                    && visit(t.getEnclosingType(), s.getEnclosingType())
                    && containsTypeEquivalent(t.getTypeArguments(), s.getTypeArguments());
            }

            @Override
            public Boolean visitArrayType(ArrayType t, Type s) {
                if (t == s)
                    return true;

                if (s.isPartial())
                    return visit(s, t);

                return s.hasTag(ARRAY)
                    && containsTypeEquivalent(t.elemtype, elemtype(s));
            }

            @Override
            public Boolean visitMethodType(MethodType t, Type s) {
                // isSameType for methods does not take thrown
                // exceptions into account!
                return hasSameArgs(t, s) && visit(t.getReturnType(), s.getReturnType());
            }

            @Override
            public Boolean visitPackageType(PackageType t, Type s) {
                return t == s;
            }

            @Override
            public Boolean visitForAll(ForAll t, Type s) {
                if (!s.hasTag(FORALL)) {
                    return false;
                }

                ForAll forAll = (ForAll)s;
                return hasSameBounds(t, forAll)
                    && visit(t.qtype, subst(forAll.qtype, forAll.tvars, t.tvars));
            }

            @Override
            public Boolean visitUndetVar(UndetVar t, Type s) {
                if (s.hasTag(WILDCARD)) {
                    // FIXME, this might be leftovers from before capture conversion
                    return false;
                }

                if (t == s || t.qtype == s || s.hasTag(ERROR) || s.hasTag(UNKNOWN)) {
                    return true;
                }

                t.addBound(InferenceBound.EQ, s, Types.this);

                return true;
            }

            @Override
            public Boolean visitErrorType(ErrorType t, Type s) {
                return true;
            }
        };
    public boolean containedBy(Type t, Type s) {
        switch (t.getTag()) {
            case UNDETVAR:
                if (s.hasTag(WILDCARD)) {
                    UndetVar undetvar = (UndetVar) t;
                    WildcardType wt = (WildcardType) s;
                    switch (wt.kind) {
                        case UNBOUND:
                            break;
                        case EXTENDS:
                            {
                                Type bound = wildUpperBound(s);
                                undetvar.addBound(InferenceBound.UPPER, bound, this);
                                break;
                            }
                        case SUPER:
                            {
                                Type bound = wildLowerBound(s);
                                undetvar.addBound(InferenceBound.LOWER, bound, this);
                                break;
                            }
                    }
                    return true;
                } else {
                    return isSameType(t, s);
                }
            case ERROR:
                return true;
            default:
                return containsType(s, t);
        }
    }
    boolean containsType(List<Type> ts, List<Type> ss) {
        while (ts.nonEmpty() && ss.nonEmpty() && containsType(ts.head, ss.head)) {
            ts = ts.tail;
            ss = ss.tail;
        }
        return ts.isEmpty() && ss.isEmpty();
    }
    public boolean containsType(Type t, Type s) {
        return containsType.visit(t, s);
    }
    private TypeRelation containsType = new TypeRelation() {

            public Boolean visitType(Type t, Type s) {
                if (s.isPartial())
                    return containedBy(s, t);
                else
                    return isSameType(t, s);
            }

//            void debugContainsType(WildcardType t, Type s) {
//                System.err.println();
//                System.err.format(" does %s contain %s?%n", t, s);
//                System.err.format(" %s U(%s) <: U(%s) %s = %s%n",
//                                  wildUpperBound(s), s, t, wildUpperBound(t),
//                                  t.isSuperBound()
//                                  || isSubtypeNoCapture(wildUpperBound(s), wildUpperBound(t)));
//                System.err.format(" %s L(%s) <: L(%s) %s = %s%n",
//                                  wildLowerBound(t), t, s, wildLowerBound(s),
//                                  t.isExtendsBound()
//                                  || isSubtypeNoCapture(wildLowerBound(t), wildLowerBound(s)));
//                System.err.println();
//            }

            @Override
            public Boolean visitWildcardType(WildcardType t, Type s) {
                if (s.isPartial())
                    return containedBy(s, t);
                else {
//                    debugContainsType(t, s);
                    return isSameWildcard(t, s)
                        || isCaptureOf(s, t)
                        || ((t.isExtendsBound() || isSubtypeNoCapture(wildLowerBound(t), wildLowerBound(s))) &&
                            (t.isSuperBound() || isSubtypeNoCapture(wildUpperBound(s), wildUpperBound(t))));
                }
            }

            @Override
            public Boolean visitUndetVar(UndetVar t, Type s) {
                if (!s.hasTag(WILDCARD)) {
                    return isSameType(t, s);
                } else {
                    return false;
                }
            }

            @Override
            public Boolean visitErrorType(ErrorType t, Type s) {
                return true;
            }
        };
    public boolean isCaptureOf(Type s, WildcardType t) {
        return !s.hasTag(TYPEVAR) || !((TypeVar) s).isCaptured() ? false : isSameWildcard(t, ((CapturedType) s).wildcard);
    }
    public boolean isSameWildcard(WildcardType t, Type s) {
        if (!s.hasTag(WILDCARD)) 
            return false;
        WildcardType w = (WildcardType) s;
        return w.kind == t.kind && w.type == t.type;
    }
    public boolean containsTypeEquivalent(List<Type> ts, List<Type> ss) {
        while (ts.nonEmpty() && ss.nonEmpty() && containsTypeEquivalent(ts.head, ss.head)) {
            ts = ts.tail;
            ss = ss.tail;
        }
        return ts.isEmpty() && ss.isEmpty();
    }
    public boolean isCastable(Type t, Type s) {
        return isCastable(t, s, noWarnings);
    }
    public boolean isCastable(Type t, Type s, Warner warn) {
        if (t == s) 
            return true;
        if (t.isPrimitive() != s.isPrimitive()) {
            t = skipTypeVars(t, false);
            return isConvertible(t, s, warn) || s.isPrimitive() && isSubtype(boxedClass(s).type, t);
        }
        boolean result;
        if (warn != warnStack.head) {
            try {
                warnStack = warnStack.prepend(warn);
                checkUnsafeVarargsConversion(t, s, warn);
                result = isCastable.visit(t, s);
            } finally {
                warnStack = warnStack.tail;
            }
        } else {
            result = isCastable.visit(t, s);
        }
        return result && t.hasTag(CLASS) && t.tsym.kind.matches(Kinds.KindSelector.TYP) && s.hasTag(CLASS) && s.tsym.kind.matches(Kinds.KindSelector.TYP) && (t.tsym.isSealed() || s.tsym.isSealed()) ? t.isCompound() || s.isCompound() || !areDisjoint((ClassSymbol) t.tsym, (ClassSymbol) s.tsym) : result;
    }
    private boolean areDisjoint(ClassSymbol ts, ClassSymbol ss) {
        if (isSubtype(erasure(ts.type), erasure(ss.type))) {
            return false;
        }
        if (ts.isInterface() == ss.isInterface() && isSubtype(erasure(ss.type), erasure(ts.type))) {
            return false;
        }
        if (ts.isInterface() && !ss.isInterface()) {
            return areDisjoint(ss, ts);
        }
        if (!ts.isInterface() && ts.isFinal()) {
            return true;
        }
        if (ts.isSealed() || ss.isSealed()) {
            ClassSymbol sealedOne = ts.isSealed() ? ts : ss;
            return sealedOne.permitted.stream().allMatch((sym) -> areDisjoint((ClassSymbol) sym, sealedOne == ts ? ss : ts));
        }
        return false;
    }
    private TypeRelation isCastable = new TypeRelation() {

            public Boolean visitType(Type t, Type s) {
                if (s.hasTag(ERROR) || t.hasTag(NONE))
                    return true;

                switch (t.getTag()) {
                case BYTE: case CHAR: case SHORT: case INT: case LONG: case FLOAT:
                case DOUBLE:
                    return s.isNumeric();
                case BOOLEAN:
                    return s.hasTag(BOOLEAN);
                case VOID:
                    return false;
                case BOT:
                    return isSubtype(t, s);
                default:
                    throw new AssertionError();
                }
            }

            @Override
            public Boolean visitWildcardType(WildcardType t, Type s) {
                return isCastable(wildUpperBound(t), s, warnStack.head);
            }

            @Override
            public Boolean visitClassType(ClassType t, Type s) {
                if (s.hasTag(ERROR) || s.hasTag(BOT))
                    return true;

                if (s.hasTag(TYPEVAR)) {
                    if (isCastable(t, s.getUpperBound(), noWarnings)) {
                        warnStack.head.warn(LintCategory.UNCHECKED);
                        return true;
                    } else {
                        return false;
                    }
                }

                if (t.isCompound() || s.isCompound()) {
                    return !t.isCompound() ?
                            visitCompoundType((ClassType)s, t, true) :
                            visitCompoundType(t, s, false);
                }

                if (s.hasTag(CLASS) || s.hasTag(ARRAY)) {
                    boolean upcast;
                    if ((upcast = isSubtype(erasure(t), erasure(s)))
                        || isSubtype(erasure(s), erasure(t))) {
                        if (!upcast && s.hasTag(ARRAY)) {
                            if (!isReifiable(s))
                                warnStack.head.warn(LintCategory.UNCHECKED);
                            return true;
                        } else if (s.isRaw()) {
                            return true;
                        } else if (t.isRaw()) {
                            if (!isUnbounded(s))
                                warnStack.head.warn(LintCategory.UNCHECKED);
                            return true;
                        }
                        // Assume |a| <: |b|
                        final Type a = upcast ? t : s;
                        final Type b = upcast ? s : t;
                        final boolean HIGH = true;
                        final boolean LOW = false;
                        final boolean DONT_REWRITE_TYPEVARS = false;
                        Type aHigh = rewriteQuantifiers(a, HIGH, DONT_REWRITE_TYPEVARS);
                        Type aLow  = rewriteQuantifiers(a, LOW,  DONT_REWRITE_TYPEVARS);
                        Type bHigh = rewriteQuantifiers(b, HIGH, DONT_REWRITE_TYPEVARS);
                        Type bLow  = rewriteQuantifiers(b, LOW,  DONT_REWRITE_TYPEVARS);
                        Type lowSub = asSub(bLow, aLow.tsym);
                        Type highSub = (lowSub == null) ? null : asSub(bHigh, aHigh.tsym);
                        if (highSub == null) {
                            final boolean REWRITE_TYPEVARS = true;
                            aHigh = rewriteQuantifiers(a, HIGH, REWRITE_TYPEVARS);
                            aLow  = rewriteQuantifiers(a, LOW,  REWRITE_TYPEVARS);
                            bHigh = rewriteQuantifiers(b, HIGH, REWRITE_TYPEVARS);
                            bLow  = rewriteQuantifiers(b, LOW,  REWRITE_TYPEVARS);
                            lowSub = asSub(bLow, aLow.tsym);
                            highSub = (lowSub == null) ? null : asSub(bHigh, aHigh.tsym);
                        }
                        if (highSub != null) {
                            if (!(a.tsym == highSub.tsym && a.tsym == lowSub.tsym)) {
                                Assert.error(a.tsym + " != " + highSub.tsym + " != " + lowSub.tsym);
                            }
                            if (!disjointTypes(aHigh.allparams(), highSub.allparams())
                                && !disjointTypes(aHigh.allparams(), lowSub.allparams())
                                && !disjointTypes(aLow.allparams(), highSub.allparams())
                                && !disjointTypes(aLow.allparams(), lowSub.allparams())) {
                                if (upcast ? giveWarning(a, b) :
                                    giveWarning(b, a))
                                    warnStack.head.warn(LintCategory.UNCHECKED);
                                return true;
                            }
                        }
                        if (isReifiable(s))
                            return isSubtypeUnchecked(a, b);
                        else
                            return isSubtypeUnchecked(a, b, warnStack.head);
                    }

                    // Sidecast
                    if (s.hasTag(CLASS)) {
                        if ((s.tsym.flags() & INTERFACE) != 0) {
                            return ((t.tsym.flags() & FINAL) == 0)
                                ? sideCast(t, s, warnStack.head)
                                : sideCastFinal(t, s, warnStack.head);
                        } else if ((t.tsym.flags() & INTERFACE) != 0) {
                            return ((s.tsym.flags() & FINAL) == 0)
                                ? sideCast(t, s, warnStack.head)
                                : sideCastFinal(t, s, warnStack.head);
                        } else {
                            // unrelated class types
                            return false;
                        }
                    }
                }
                return false;
            }

            boolean visitCompoundType(ClassType ct, Type s, boolean reverse) {
                Warner warn = noWarnings;
                for (Type c : directSupertypes(ct)) {
                    warn.clear();
                    if (reverse ? !isCastable(s, c, warn) : !isCastable(c, s, warn))
                        return false;
                }
                if (warn.hasLint(LintCategory.UNCHECKED))
                    warnStack.head.warn(LintCategory.UNCHECKED);
                return true;
            }

            @Override
            public Boolean visitArrayType(ArrayType t, Type s) {
                switch (s.getTag()) {
                case ERROR:
                case BOT:
                    return true;
                case TYPEVAR:
                    if (isCastable(s, t, noWarnings)) {
                        warnStack.head.warn(LintCategory.UNCHECKED);
                        return true;
                    } else {
                        return false;
                    }
                case CLASS:
                    return isSubtype(t, s);
                case ARRAY:
                    if (elemtype(t).isPrimitive() || elemtype(s).isPrimitive()) {
                        return elemtype(t).hasTag(elemtype(s).getTag());
                    } else {
                        return visit(elemtype(t), elemtype(s));
                    }
                default:
                    return false;
                }
            }

            @Override
            public Boolean visitTypeVar(TypeVar t, Type s) {
                switch (s.getTag()) {
                case ERROR:
                case BOT:
                    return true;
                case TYPEVAR:
                    if (isSubtype(t, s)) {
                        return true;
                    } else if (isCastable(t.getUpperBound(), s, noWarnings)) {
                        warnStack.head.warn(LintCategory.UNCHECKED);
                        return true;
                    } else {
                        return false;
                    }
                default:
                    return isCastable(t.getUpperBound(), s, warnStack.head);
                }
            }

            @Override
            public Boolean visitErrorType(ErrorType t, Type s) {
                return true;
            }
        };
    public boolean disjointTypes(List<Type> ts, List<Type> ss) {
        while (ts.tail != null && ss.tail != null) {
            if (disjointType(ts.head, ss.head)) 
                return true;
            ts = ts.tail;
            ss = ss.tail;
        }
        return false;
    }
    public boolean disjointType(Type t, Type s) {
        return disjointType.visit(t, s);
    }
    private TypeRelation disjointType = new TypeRelation() {

            private Set<TypePair> cache = new HashSet<>();

            @Override
            public Boolean visitType(Type t, Type s) {
                if (s.hasTag(WILDCARD))
                    return visit(s, t);
                else
                    return notSoftSubtypeRecursive(t, s) || notSoftSubtypeRecursive(s, t);
            }

            private boolean isCastableRecursive(Type t, Type s) {
                TypePair pair = new TypePair(t, s);
                if (cache.add(pair)) {
                    try {
                        return Types.this.isCastable(t, s);
                    } finally {
                        cache.remove(pair);
                    }
                } else {
                    return true;
                }
            }

            private boolean notSoftSubtypeRecursive(Type t, Type s) {
                TypePair pair = new TypePair(t, s);
                if (cache.add(pair)) {
                    try {
                        return Types.this.notSoftSubtype(t, s);
                    } finally {
                        cache.remove(pair);
                    }
                } else {
                    return false;
                }
            }

            @Override
            public Boolean visitWildcardType(WildcardType t, Type s) {
                if (t.isUnbound())
                    return false;

                if (!s.hasTag(WILDCARD)) {
                    if (t.isExtendsBound())
                        return notSoftSubtypeRecursive(s, t.type);
                    else
                        return notSoftSubtypeRecursive(t.type, s);
                }

                if (s.isUnbound())
                    return false;

                if (t.isExtendsBound()) {
                    if (s.isExtendsBound())
                        return !isCastableRecursive(t.type, wildUpperBound(s));
                    else if (s.isSuperBound())
                        return notSoftSubtypeRecursive(wildLowerBound(s), t.type);
                } else if (t.isSuperBound()) {
                    if (s.isExtendsBound())
                        return notSoftSubtypeRecursive(t.type, wildUpperBound(s));
                }
                return false;
            }
        };
    public List<Type> cvarLowerBounds(List<Type> ts) {
        return ts.map(cvarLowerBoundMapping);
    }
    private final TypeMapping<Void> cvarLowerBoundMapping = new TypeMapping<Void>() {
            @Override
            public Type visitCapturedType(CapturedType t, Void _unused) {
                return cvarLowerBound(t);
            }
        };
    public boolean notSoftSubtype(Type t, Type s) {
        if (t == s) 
            return false;
        if (t.hasTag(TYPEVAR)) {
            return !isCastable(((TypeVar) t).getUpperBound(), relaxBound(s), noWarnings);
        }
        if (!s.hasTag(WILDCARD)) 
            s = cvarUpperBound(s);
        return !isSubtype(t, relaxBound(s));
    }
    private Type relaxBound(Type t) {
        return t.hasTag(TYPEVAR) ? rewriteQuantifiers(skipTypeVars(t, false), true, true) : t;
    }
    public boolean isReifiable(Type t) {
        return isReifiable.visit(t);
    }
    private UnaryVisitor<Boolean> isReifiable = new UnaryVisitor<Boolean>() {

            public Boolean visitType(Type t, Void ignored) {
                return true;
            }

            @Override
            public Boolean visitClassType(ClassType t, Void ignored) {
                if (t.isCompound())
                    return false;
                else {
                    if (!t.isParameterized())
                        return true;

                    for (Type param : t.allparams()) {
                        if (!param.isUnbound())
                            return false;
                    }
                    return true;
                }
            }

            @Override
            public Boolean visitArrayType(ArrayType t, Void ignored) {
                return visit(t.elemtype);
            }

            @Override
            public Boolean visitTypeVar(TypeVar t, Void ignored) {
                return false;
            }
        };
    public boolean isArray(Type t) {
        while (t.hasTag(WILDCARD)) 
            t = wildUpperBound(t);
        return t.hasTag(ARRAY);
    }
    public Type elemtype(Type t) {
        switch (t.getTag()) {
            case WILDCARD:
                return elemtype(wildUpperBound(t));
            case ARRAY:
                return ((ArrayType) t).elemtype;
            case FORALL:
                return elemtype(((ForAll) t).qtype);
            case ERROR:
                return t;
            default:
                return null;
        }
    }
    public Type elemtypeOrType(Type t) {
        Type elemtype = elemtype(t);
        return elemtype != null ? elemtype : t;
    }
    private TypeMapping<Void> elemTypeFun = new TypeMapping<Void>() {
        @Override
        public Type visitArrayType(ArrayType t, Void _unused) {
            return t.elemtype;
        }

        @Override
        public Type visitTypeVar(TypeVar t, Void _unused) {
            return visit(skipTypeVars(t, false));
        }
    };
    public int dimensions(Type t) {
        int result = 0;
        while (t.hasTag(ARRAY)) {
            result++;
            t = elemtype(t);
        }
        return result;
    }
    public ArrayType makeArrayType(Type t) {
        if (t.hasTag(VOID) || t.hasTag(PACKAGE)) {
            Assert.error("Type t must not be a VOID or PACKAGE type, " + t.toString());
        }
        return new ArrayType(t, syms.arrayClass);
    }
    public Type asSuper(Type t, Symbol sym) {
        return sym.type == syms.objectType ? syms.objectType : asSuper.visit(t, sym);
    }
    private SimpleVisitor<Type,Symbol> asSuper = new SimpleVisitor<Type,Symbol>() {

            private Set<Symbol> seenTypes = new HashSet<>();

            public Type visitType(Type t, Symbol sym) {
                return null;
            }

            @Override
            public Type visitClassType(ClassType t, Symbol sym) {
                if (t.tsym == sym)
                    return t;

                Symbol c = t.tsym;
                if (!seenTypes.add(c)) {
                    return null;
                }
                try {
                    Type st = supertype(t);
                    if (st.hasTag(CLASS) || st.hasTag(TYPEVAR)) {
                        Type x = asSuper(st, sym);
                        if (x != null)
                            return x;
                    }
                    if ((sym.flags() & INTERFACE) != 0) {
                        for (List<Type> l = interfaces(t); l.nonEmpty(); l = l.tail) {
                            if (!l.head.hasTag(ERROR)) {
                                Type x = asSuper(l.head, sym);
                                if (x != null)
                                    return x;
                            }
                        }
                    }
                    return null;
                } finally {
                    seenTypes.remove(c);
                }
            }

            @Override
            public Type visitArrayType(ArrayType t, Symbol sym) {
                return isSubtype(t, sym.type) ? sym.type : null;
            }

            @Override
            public Type visitTypeVar(TypeVar t, Symbol sym) {
                if (t.tsym == sym)
                    return t;
                else
                    return asSuper(t.getUpperBound(), sym);
            }

            @Override
            public Type visitErrorType(ErrorType t, Symbol sym) {
                return t;
            }
        };
    public Type asOuterSuper(Type t, Symbol sym) {
        switch (t.getTag()) {
            case CLASS:
                do {
                    Type s = asSuper(t, sym);
                    if (s != null) 
                        return s;
                    t = t.getEnclosingType();
                } while (t.hasTag(CLASS));
                return null;
            case ARRAY:
                return isSubtype(t, sym.type) ? sym.type : null;
            case TYPEVAR:
                return asSuper(t, sym);
            case ERROR:
                return t;
            default:
                return null;
        }
    }
    public Type asEnclosingSuper(Type t, Symbol sym) {
        switch (t.getTag()) {
            case CLASS:
                do {
                    Type s = asSuper(t, sym);
                    if (s != null) 
                        return s;
                    Type outer = t.getEnclosingType();
                    t = outer.hasTag(CLASS) ? outer : t.tsym.owner.enclClass() != null ? t.tsym.owner.enclClass().type : Type.noType;
                } while (t.hasTag(CLASS));
                return null;
            case ARRAY:
                return isSubtype(t, sym.type) ? sym.type : null;
            case TYPEVAR:
                return asSuper(t, sym);
            case ERROR:
                return t;
            default:
                return null;
        }
    }
    public Type memberType(Type t, Symbol sym) {
        return (sym.flags() & STATIC) != 0 ? sym.type : memberType.visit(t, sym);
    }
    private SimpleVisitor<Type,Symbol> memberType = new SimpleVisitor<Type,Symbol>() {

            public Type visitType(Type t, Symbol sym) {
                return sym.type;
            }

            @Override
            public Type visitWildcardType(WildcardType t, Symbol sym) {
                return memberType(wildUpperBound(t), sym);
            }

            @Override
            public Type visitClassType(ClassType t, Symbol sym) {
                Symbol owner = sym.owner;
                long flags = sym.flags();
                if (((flags & STATIC) == 0) && owner.type.isParameterized()) {
                    Type base = asOuterSuper(t, owner);
                    //if t is an intersection type T = CT & I1 & I2 ... & In
                    //its supertypes CT, I1, ... In might contain wildcards
                    //so we need to go through capture conversion
                    base = t.isCompound() ? capture(base) : base;
                    if (base != null) {
                        List<Type> ownerParams = owner.type.allparams();
                        List<Type> baseParams = base.allparams();
                        if (ownerParams.nonEmpty()) {
                            if (baseParams.isEmpty()) {
                                // then base is a raw type
                                return erasure(sym.type);
                            } else {
                                return subst(sym.type, ownerParams, baseParams);
                            }
                        }
                    }
                }
                return sym.type;
            }

            @Override
            public Type visitTypeVar(TypeVar t, Symbol sym) {
                return memberType(t.getUpperBound(), sym);
            }

            @Override
            public Type visitErrorType(ErrorType t, Symbol sym) {
                return t;
            }
        };
    public boolean isAssignable(Type t, Type s) {
        return isAssignable(t, s, noWarnings);
    }
    public boolean isAssignable(Type t, Type s, Warner warn) {
        if (t.hasTag(ERROR)) 
            return true;
        if (t.getTag().isSubRangeOf(INT) && t.constValue() != null) {
            int value = ((Number) t.constValue()).intValue();
            switch (s.getTag()) {
                case BYTE:
                case CHAR:
                case SHORT:
                case INT:
                    if (s.getTag().checkRange(value)) 
                        return true;
                    break;
                case CLASS:
                    switch (unboxedType(s).getTag()) {
                        case BYTE:
                        case CHAR:
                        case SHORT:
                            return isAssignable(t, unboxedType(s), warn);
                    }
                    break;
            }
        }
        return isConvertible(t, s, warn);
    }
    public Type erasure(Type t) {
        return eraseNotNeeded(t) ? t : erasure(t, false);
    }
    private boolean eraseNotNeeded(Type t) {
        return t.isPrimitive() || syms.stringType.tsym == t.tsym;
    }
    private Type erasure(Type t, boolean recurse) {
        return t.isPrimitive() ? t : erasure.visit(t, recurse);
    }
    private TypeMapping<Boolean> erasure = new StructuralTypeMapping<Boolean>() {
            private Type combineMetadata(final Type s,
                                         final Type t) {
                if (t.getMetadata() != TypeMetadata.EMPTY) {
                    switch (s.getKind()) {
                        case OTHER:
                        case UNION:
                        case INTERSECTION:
                        case PACKAGE:
                        case EXECUTABLE:
                        case NONE:
                        case VOID:
                        case ERROR:
                            return s;
                        default: return s.cloneWithMetadata(s.getMetadata().without(Kind.ANNOTATIONS));
                    }
                } else {
                    return s;
                }
            }

            public Type visitType(Type t, Boolean recurse) {
                if (t.isPrimitive())
                    return t; /*fast special case*/
                else {
                    //other cases already handled
                    return combineMetadata(t, t);
                }
            }

            @Override
            public Type visitWildcardType(WildcardType t, Boolean recurse) {
                Type erased = erasure(wildUpperBound(t), recurse);
                return combineMetadata(erased, t);
            }

            @Override
            public Type visitClassType(ClassType t, Boolean recurse) {
                Type erased = t.tsym.erasure(Types.this);
                if (recurse) {
                    erased = new ErasedClassType(erased.getEnclosingType(),erased.tsym,
                            t.getMetadata().without(Kind.ANNOTATIONS));
                    return erased;
                } else {
                    return combineMetadata(erased, t);
                }
            }

            @Override
            public Type visitTypeVar(TypeVar t, Boolean recurse) {
                Type erased = erasure(t.getUpperBound(), recurse);
                return combineMetadata(erased, t);
            }
        };
    public List<Type> erasure(List<Type> ts) {
        return erasure.visit(ts, false);
    }
    public Type erasureRecursive(Type t) {
        return erasure(t, true);
    }
    public List<Type> erasureRecursive(List<Type> ts) {
        return erasure.visit(ts, true);
    }
    public IntersectionClassType makeIntersectionType(List<Type> bounds) {
        return makeIntersectionType(bounds, bounds.head.tsym.isInterface());
    }
    public IntersectionClassType makeIntersectionType(List<Type> bounds, boolean allInterfaces) {
        Assert.check(bounds.nonEmpty());
        Type firstExplicitBound = bounds.head;
        if (allInterfaces) {
            bounds = bounds.prepend(syms.objectType);
        }
        ClassSymbol bc = new ClassSymbol(ABSTRACT | PUBLIC | SYNTHETIC | COMPOUND | ACYCLIC, Type.moreInfo ? names.fromString(bounds.toString()) : names.empty, null, syms.noSymbol);
        IntersectionClassType intersectionType = new IntersectionClassType(bounds, bc, allInterfaces);
        bc.type = intersectionType;
        bc.erasure_field = bounds.head.hasTag(TYPEVAR) ? syms.objectType : erasure(firstExplicitBound);
        bc.members_field = WriteableScope.create(bc);
        return intersectionType;
    }
    public Type supertype(Type t) {
        return supertype.visit(t);
    }
    private UnaryVisitor<Type> supertype = new UnaryVisitor<Type>() {

            public Type visitType(Type t, Void ignored) {
                // A note on wildcards: there is no good way to
                // determine a supertype for a lower-bounded wildcard.
                return Type.noType;
            }

            @Override
            public Type visitClassType(ClassType t, Void ignored) {
                if (t.supertype_field == null) {
                    Type supertype = ((ClassSymbol)t.tsym).getSuperclass();
                    // An interface has no superclass; its supertype is Object.
                    if (t.isInterface())
                        supertype = ((ClassType)t.tsym.type).supertype_field;
                    if (t.supertype_field == null) {
                        List<Type> actuals = classBound(t).allparams();
                        List<Type> formals = t.tsym.type.allparams();
                        if (t.hasErasedSupertypes()) {
                            t.supertype_field = erasureRecursive(supertype);
                        } else if (formals.nonEmpty()) {
                            t.supertype_field = subst(supertype, formals, actuals);
                        }
                        else {
                            t.supertype_field = supertype;
                        }
                    }
                }
                return t.supertype_field;
            }

            /**
             * The supertype is always a class type. If the type
             * variable's bounds start with a class type, this is also
             * the supertype.  Otherwise, the supertype is
             * java.lang.Object.
             */
            @Override
            public Type visitTypeVar(TypeVar t, Void ignored) {
                if (t.getUpperBound().hasTag(TYPEVAR) ||
                    (!t.getUpperBound().isCompound() && !t.getUpperBound().isInterface())) {
                    return t.getUpperBound();
                } else {
                    return supertype(t.getUpperBound());
                }
            }

            @Override
            public Type visitArrayType(ArrayType t, Void ignored) {
                if (t.elemtype.isPrimitive() || isSameType(t.elemtype, syms.objectType))
                    return arraySuperType();
                else
                    return new ArrayType(supertype(t.elemtype), t.tsym);
            }

            @Override
            public Type visitErrorType(ErrorType t, Void ignored) {
                return Type.noType;
            }
        };
    public List<Type> interfaces(Type t) {
        return interfaces.visit(t);
    }
    private UnaryVisitor<List<Type>> interfaces = new UnaryVisitor<List<Type>>() {

            public List<Type> visitType(Type t, Void ignored) {
                return List.nil();
            }

            @Override
            public List<Type> visitClassType(ClassType t, Void ignored) {
                if (t.interfaces_field == null) {
                    List<Type> interfaces = ((ClassSymbol)t.tsym).getInterfaces();
                    if (t.interfaces_field == null) {
                        // If t.interfaces_field is null, then t must
                        // be a parameterized type (not to be confused
                        // with a generic type declaration).
                        // Terminology:
                        //    Parameterized type: List<String>
                        //    Generic type declaration: class List<E> { ... }
                        // So t corresponds to List<String> and
                        // t.tsym.type corresponds to List<E>.
                        // The reason t must be parameterized type is
                        // that completion will happen as a side
                        // effect of calling
                        // ClassSymbol.getInterfaces.  Since
                        // t.interfaces_field is null after
                        // completion, we can assume that t is not the
                        // type of a class/interface declaration.
                        Assert.check(t != t.tsym.type, t);
                        List<Type> actuals = t.allparams();
                        List<Type> formals = t.tsym.type.allparams();
                        if (t.hasErasedSupertypes()) {
                            t.interfaces_field = erasureRecursive(interfaces);
                        } else if (formals.nonEmpty()) {
                            t.interfaces_field = subst(interfaces, formals, actuals);
                        }
                        else {
                            t.interfaces_field = interfaces;
                        }
                    }
                }
                return t.interfaces_field;
            }

            @Override
            public List<Type> visitTypeVar(TypeVar t, Void ignored) {
                if (t.getUpperBound().isCompound())
                    return interfaces(t.getUpperBound());

                if (t.getUpperBound().isInterface())
                    return List.of(t.getUpperBound());

                return List.nil();
            }
        };
    public List<Type> directSupertypes(Type t) {
        return directSupertypes.visit(t);
    }
    private final UnaryVisitor<List<Type>> directSupertypes = new UnaryVisitor<List<Type>>() {

            public List<Type> visitType(final Type type, final Void ignored) {
                if (!type.isIntersection()) {
                    final Type sup = supertype(type);
                    return (sup == Type.noType || sup == type || sup == null)
                        ? interfaces(type)
                        : interfaces(type).prepend(sup);
                } else {
                    return ((IntersectionClassType)type).getExplicitComponents();
                }
            }
        };
    public boolean isDirectSuperInterface(TypeSymbol isym, TypeSymbol origin) {
        for (Type i2 : interfaces(origin.type)) {
            if (isym == i2.tsym) 
                return true;
        }
        return false;
    }
    Map<Type,Boolean> isDerivedRawCache = new HashMap<>();
    public boolean isDerivedRaw(Type t) {
        Boolean result = isDerivedRawCache.get(t);
        if (result == null) {
            result = isDerivedRawInternal(t);
            isDerivedRawCache.put(t, result);
        }
        return result;
    }
    public boolean isDerivedRawInternal(Type t) {
        return t.isErroneous() ? false : t.isRaw() || supertype(t) != Type.noType && isDerivedRaw(supertype(t)) || isDerivedRaw(interfaces(t));
    }
    public boolean isDerivedRaw(List<Type> ts) {
        List<Type> l = ts;
        while (l.nonEmpty() && !isDerivedRaw(l.head)) 
            l = l.tail;
        return l.nonEmpty();
    }
    public void setBounds(TypeVar t, List<Type> bounds) {
        setBounds(t, bounds, bounds.head.tsym.isInterface());
    }
    public void setBounds(TypeVar t, List<Type> bounds, boolean allInterfaces) {
        t.setUpperBound(bounds.tail.isEmpty() ? bounds.head : makeIntersectionType(bounds, allInterfaces));
        t.rank_field = -1;
    }
    public List<Type> getBounds(TypeVar t) {
        return t.getUpperBound().hasTag(NONE) ? List.nil() : t.getUpperBound().isErroneous() || !t.getUpperBound().isCompound() ? List.of(t.getUpperBound()) : (erasure(t).tsym.flags() & INTERFACE) == 0 ? interfaces(t).prepend(supertype(t)) : interfaces(t);
    }
    public Type classBound(Type t) {
        return classBound.visit(t);
    }
    private UnaryVisitor<Type> classBound = new UnaryVisitor<Type>() {

            public Type visitType(Type t, Void ignored) {
                return t;
            }

            @Override
            public Type visitClassType(ClassType t, Void ignored) {
                Type outer1 = classBound(t.getEnclosingType());
                if (outer1 != t.getEnclosingType())
                    return new ClassType(outer1, t.getTypeArguments(), t.tsym,
                                         t.getMetadata());
                else
                    return t;
            }

            @Override
            public Type visitTypeVar(TypeVar t, Void ignored) {
                return classBound(supertype(t));
            }

            @Override
            public Type visitErrorType(ErrorType t, Void ignored) {
                return t;
            }
        };
    public boolean isSubSignature(Type t, Type s) {
        return isSubSignature(t, s, true);
    }
    public boolean isSubSignature(Type t, Type s, boolean strict) {
        return hasSameArgs(t, s, strict) || hasSameArgs(t, erasure(s), strict);
    }
    public boolean overrideEquivalent(Type t, Type s) {
        return hasSameArgs(t, s) || hasSameArgs(t, erasure(s)) || hasSameArgs(erasure(t), s);
    }
    public boolean overridesObjectMethod(TypeSymbol origin, Symbol msym) {
        for (Symbol sym : syms.objectType.tsym.members().getSymbolsByName(msym.name)) {
            if (msym.overrides(sym, origin, Types.this, true)) {
                return true;
            }
        }
        return false;
    }
    public enum MostSpecificReturnCheck {
        BASIC() {
            @Override
            public boolean test(Type mt1, Type mt2, Types types) {
                List<Type> tvars = mt1.getTypeArguments();
                List<Type> svars = mt2.getTypeArguments();
                Type t = mt1.getReturnType();
                Type s = types.subst(mt2.getReturnType(), svars, tvars);
                return types.isSameType(t, s) ||
                    !t.isPrimitive() &&
                    !s.isPrimitive() &&
                    types.isSubtype(t, s);
            }
        }, RTS() {
            @Override
            public boolean test(Type mt1, Type mt2, Types types) {
                return types.returnTypeSubstitutable(mt1, mt2);
            }
        };
        public abstract boolean test(Type mt1, Type mt2, Types types);
    }
    public Optional<Symbol> mergeAbstracts(List<Symbol> ambiguousInOrder, Type site, boolean sigCheck) {
        boolean shouldErase = false;
        List<Type> erasedParams = ambiguousInOrder.head.erasure(this).getParameterTypes();
        for (Symbol s : ambiguousInOrder) {
            if ((s.flags() & ABSTRACT) == 0 || sigCheck && !isSameTypes(erasedParams, s.erasure(this).getParameterTypes())) {
                return Optional.empty();
            } else if (s.type.hasTag(FORALL)) {
                shouldErase = true;
            }
        }
        for (MostSpecificReturnCheck mostSpecificReturnCheck : MostSpecificReturnCheck.values()) {
            outer:
                for (Symbol s : ambiguousInOrder) {
                    Type mt = memberType(site, s);
                    List<Type> allThrown = mt.getThrownTypes();
                    for (Symbol s2 : ambiguousInOrder) {
                        if (s != s2) {
                            Type mt2 = memberType(site, s2);
                            if (!isSubSignature(mt, mt2) || !mostSpecificReturnCheck.test(mt, mt2, this)) {
                                continue outer;
                            } else {
                                List<Type> thrownTypes2 = mt2.getThrownTypes();
                                if (!mt.hasTag(FORALL) && shouldErase) {
                                    thrownTypes2 = erasure(thrownTypes2);
                                } else if (mt.hasTag(FORALL)) {
                                    Assert.check(mt2.hasTag(FORALL));
                                    thrownTypes2 = subst(thrownTypes2, mt2.getTypeArguments(), mt.getTypeArguments());
                                }
                                allThrown = chk.intersect(allThrown, thrownTypes2);
                            }
                        }
                    }
                    return allThrown == mt.getThrownTypes() ? Optional.of(s) : Optional.of(new MethodSymbol(s.flags(), s.name, createMethodTypeWithThrown(s.type, allThrown), s.owner) {
                            @Override
                            public Symbol baseSymbol() {
                                return s;
                            }
                        });
                }
        }
        return Optional.empty();
    }
    class ImplementationCache {
        private WeakHashMap<MethodSymbol, SoftReference<Map<TypeSymbol, Entry>>> _map = new WeakHashMap<>();
        class Entry {
            final MethodSymbol cachedImpl;
            final Predicate<Symbol> implFilter;
            final boolean checkResult;
            final int prevMark;
            public Entry(MethodSymbol cachedImpl, Predicate<Symbol> scopeFilter, boolean checkResult, int prevMark) {
                this.cachedImpl = cachedImpl;
                this.implFilter = scopeFilter;
                this.checkResult = checkResult;
                this.prevMark = prevMark;
            }
            boolean matches(Predicate<Symbol> scopeFilter, boolean checkResult, int mark) {
                return this.implFilter == scopeFilter && this.checkResult == checkResult && this.prevMark == mark;
            }
        }
        MethodSymbol get(MethodSymbol ms, TypeSymbol origin, boolean checkResult, Predicate<Symbol> implFilter) {
            SoftReference<Map<TypeSymbol, Entry>> ref_cache = _map.get(ms);
            Map<TypeSymbol, Entry> cache = ref_cache != null ? ref_cache.get() : null;
            if (cache == null) {
                cache = new HashMap<>();
                _map.put(ms, new SoftReference<>(cache));
            }
            Entry e = cache.get(origin);
            CompoundScope members = membersClosure(origin.type, true);
            if (e == null || !e.matches(implFilter, checkResult, members.getMark())) {
                MethodSymbol impl = implementationInternal(ms, origin, checkResult, implFilter);
                cache.put(origin, new Entry(impl, implFilter, checkResult, members.getMark()));
                return impl;
            } else {
                return e.cachedImpl;
            }
        }
        private MethodSymbol implementationInternal(MethodSymbol ms, TypeSymbol origin, boolean checkResult, Predicate<Symbol> implFilter) {
            for (Type t = origin.type; t.hasTag(CLASS) || t.hasTag(TYPEVAR); t = supertype(t)) {
                t = skipTypeVars(t, false);
                TypeSymbol c = t.tsym;
                Symbol bestSoFar = null;
                for (Symbol sym : c.members().getSymbolsByName(ms.name, implFilter)) {
                    if (sym != null && sym.overrides(ms, origin, Types.this, checkResult)) {
                        bestSoFar = sym;
                        if ((sym.flags() & ABSTRACT) == 0) {
                            break;
                        }
                    }
                }
                if (bestSoFar != null) {
                    return (MethodSymbol) bestSoFar;
                }
            }
            return null;
        }
    }
    private ImplementationCache implCache = new ImplementationCache();
    public MethodSymbol implementation(MethodSymbol ms, TypeSymbol origin, boolean checkResult, Predicate<Symbol> implFilter) {
        return implCache.get(ms, origin, checkResult, implFilter);
    }
    class MembersClosureCache extends SimpleVisitor<Scope.CompoundScope, Void> {
        private Map<TypeSymbol, CompoundScope> _map = new HashMap<>();
        Set<TypeSymbol> seenTypes = new HashSet<>();
        class MembersScope extends CompoundScope {
            CompoundScope scope;
            public MembersScope(CompoundScope scope) {
                super(scope.owner);
                this.scope = scope;
            }
            Predicate<Symbol> combine(Predicate<Symbol> sf) {
                return (s) -> !s.owner.isInterface() && (sf == null || sf.test(s));
            }
            @Override
            public Iterable<Symbol> getSymbols(Predicate<Symbol> sf, LookupKind lookupKind) {
                return scope.getSymbols(combine(sf), lookupKind);
            }
            @Override
            public Iterable<Symbol> getSymbolsByName(Name name, Predicate<Symbol> sf, LookupKind lookupKind) {
                return scope.getSymbolsByName(name, combine(sf), lookupKind);
            }
            @Override
            public int getMark() {
                return scope.getMark();
            }
        }
        CompoundScope nilScope;
        public CompoundScope visitType(Type t, Void _unused) {
            if (nilScope == null) {
                nilScope = new CompoundScope(syms.noSymbol);
            }
            return nilScope;
        }
        @Override
        public CompoundScope visitClassType(ClassType t, Void _unused) {
            if (!seenTypes.add(t.tsym)) {
                return new CompoundScope(t.tsym);
            }
            try {
                seenTypes.add(t.tsym);
                ClassSymbol csym = (ClassSymbol) t.tsym;
                CompoundScope membersClosure = _map.get(csym);
                if (membersClosure == null) {
                    membersClosure = new CompoundScope(csym);
                    for (Type i : interfaces(t)) {
                        membersClosure.prependSubScope(visit(i, null));
                    }
                    membersClosure.prependSubScope(visit(supertype(t), null));
                    membersClosure.prependSubScope(csym.members());
                    _map.put(csym, membersClosure);
                }
                return membersClosure;
            } finally {
                seenTypes.remove(t.tsym);
            }
        }
        @Override
        public CompoundScope visitTypeVar(TypeVar t, Void _unused) {
            return visit(t.getUpperBound(), null);
        }
    }
    private MembersClosureCache membersCache = new MembersClosureCache();
    public CompoundScope membersClosure(Type site, boolean skipInterface) {
        CompoundScope cs = membersCache.visit(site, null);
        Assert.checkNonNull(cs, () -> "type " + site);
        return skipInterface ? membersCache.new MembersScope(cs) : cs;
    }
    public MethodSymbol firstUnimplementedAbstract(ClassSymbol sym) {
        try {
            return firstUnimplementedAbstractImpl(sym, sym);
        } catch (CompletionFailure ex) {
            chk.completionError(enter.getEnv(sym).tree.pos(), ex);
            return null;
        }
    }
    private MethodSymbol firstUnimplementedAbstractImpl(ClassSymbol impl, ClassSymbol c) {
        MethodSymbol undef = null;
        if (c == impl || (c.flags() & (ABSTRACT | INTERFACE)) != 0) {
            Scope s = c.members();
            for (Symbol sym : s.getSymbols(NON_RECURSIVE)) {
                if (sym.kind == MTH && (sym.flags() & (ABSTRACT | DEFAULT | PRIVATE)) == ABSTRACT) {
                    MethodSymbol absmeth = (MethodSymbol) sym;
                    MethodSymbol implmeth = absmeth.implementation(impl, this, true);
                    if (implmeth == null || implmeth == absmeth) {
                        if (allowDefaultMethods) {
                            MethodSymbol prov = interfaceCandidates(impl.type, absmeth).head;
                            if (prov != null && prov.overrides(absmeth, impl, this, true)) {
                                implmeth = prov;
                            }
                        }
                    }
                    if (implmeth == null || implmeth == absmeth) {
                        undef = absmeth;
                        break;
                    }
                }
            }
            if (undef == null) {
                Type st = supertype(c.type);
                if (st.hasTag(CLASS)) 
                    undef = firstUnimplementedAbstractImpl(impl, (ClassSymbol) st.tsym);
            }
            for (List<Type> l = interfaces(c.type); undef == null && l.nonEmpty(); l = l.tail) {
                undef = firstUnimplementedAbstractImpl(impl, (ClassSymbol) l.head.tsym);
            }
        }
        return undef;
    }
    public class CandidatesCache {
        public Map<Entry, List<MethodSymbol>> cache = new WeakHashMap<>();
        class Entry {
            Type site;
            MethodSymbol msym;
            Entry(Type site, MethodSymbol msym) {
                this.site = site;
                this.msym = msym;
            }
            @Override
            public boolean equals(Object obj) {
                return obj instanceof Entry entry && entry.msym == msym && isSameType(site, entry.site);
            }
            @Override
            public int hashCode() {
                return Types.this.hashCode(site) & ~msym.hashCode();
            }
        }
        public List<MethodSymbol> get(Entry e) {
            return cache.get(e);
        }
        public void put(Entry e, List<MethodSymbol> msymbols) {
            cache.put(e, msymbols);
        }
    }
    public CandidatesCache candidatesCache = new CandidatesCache();
    public List<MethodSymbol> interfaceCandidates(Type site, MethodSymbol ms) {
        CandidatesCache.Entry e = candidatesCache.new Entry(site, ms);
        List<MethodSymbol> candidates = candidatesCache.get(e);
        if (candidates == null) {
            Predicate<Symbol> filter = new MethodFilter(ms, site);
            List<MethodSymbol> candidates2 = List.nil();
            for (Symbol s : membersClosure(site, false).getSymbols(filter)) {
                if (!site.tsym.isInterface() && !s.owner.isInterface()) {
                    return List.of((MethodSymbol) s);
                } else if (!candidates2.contains(s)) {
                    candidates2 = candidates2.prepend((MethodSymbol) s);
                }
            }
            candidates = prune(candidates2);
            candidatesCache.put(e, candidates);
        }
        return candidates;
    }
    public List<MethodSymbol> prune(List<MethodSymbol> methods) {
        ListBuffer<MethodSymbol> methodsMin = new ListBuffer<>();
        for (MethodSymbol m1 : methods) {
            boolean isMin_m1 = true;
            for (MethodSymbol m2 : methods) {
                if (m1 == m2) 
                    continue;
                if (m2.owner != m1.owner && asSuper(m2.owner.type, m1.owner) != null) {
                    isMin_m1 = false;
                    break;
                }
            }
            if (isMin_m1) 
                methodsMin.append(m1);
        }
        return methodsMin.toList();
    }
    private class MethodFilter implements Predicate<Symbol> {
        Symbol msym;
        Type site;
        MethodFilter(Symbol msym, Type site) {
            this.msym = msym;
            this.site = site;
        }
        @Override
                public boolean test(Symbol s) {
            return s.kind == MTH && s.name == msym.name && (s.flags() & SYNTHETIC) == 0 && s.isInheritedIn(site.tsym, Types.this) && overrideEquivalent(memberType(site, s), memberType(site, msym));
        }
    }
    public boolean hasSameArgs(Type t, Type s) {
        return hasSameArgs(t, s, true);
    }
    public boolean hasSameArgs(Type t, Type s, boolean strict) {
        return hasSameArgs(t, s, strict ? hasSameArgs_strict : hasSameArgs_nonstrict);
    }
    private boolean hasSameArgs(Type t, Type s, TypeRelation hasSameArgs) {
        return hasSameArgs.visit(t, s);
    }
    private class HasSameArgs extends TypeRelation {
        boolean strict;
        public HasSameArgs(boolean strict) {
            this.strict = strict;
        }
        public Boolean visitType(Type t, Type s) {
            throw new AssertionError();
        }
        @Override
            public Boolean visitMethodType(MethodType t, Type s) {
            return s.hasTag(METHOD) && containsTypeEquivalent(t.argtypes, s.getParameterTypes());
        }
        @Override
            public Boolean visitForAll(ForAll t, Type s) {
            if (!s.hasTag(FORALL)) 
                return strict ? false : visitMethodType(t.asMethodType(), s);
            ForAll forAll = (ForAll) s;
            return hasSameBounds(t, forAll) && visit(t.qtype, subst(forAll.qtype, forAll.tvars, t.tvars));
        }
        @Override
            public Boolean visitErrorType(ErrorType t, Type s) {
            return false;
        }
    }
    TypeRelation hasSameArgs_strict = new HasSameArgs(true);
    TypeRelation hasSameArgs_nonstrict = new HasSameArgs(false);
    public List<Type> subst(List<Type> ts, List<Type> from, List<Type> to) {
        return ts.map(new Subst(from, to));
    }
    public Type subst(Type t, List<Type> from, List<Type> to) {
        return t.map(new Subst(from, to));
    }
    private class Subst extends StructuralTypeMapping<Void> {
        List<Type> from;
        List<Type> to;
        public Subst(List<Type> from, List<Type> to) {
            int fromLength = from.length();
            int toLength = to.length();
            while (fromLength > toLength) {
                fromLength--;
                from = from.tail;
            }
            while (fromLength < toLength) {
                toLength--;
                to = to.tail;
            }
            this.from = from;
            this.to = to;
        }
        @Override
        public Type visitTypeVar(TypeVar t, Void ignored) {
            for (List<Type> from = this.from, to = this.to; from.nonEmpty(); from = from.tail, to = to.tail) {
                if (t.equalsIgnoreMetadata(from.head)) {
                    return to.head.withTypeVar(t);
                }
            }
            return t;
        }
        @Override
        public Type visitClassType(ClassType t, Void ignored) {
            if (!t.isCompound()) {
                return super.visitClassType(t, ignored);
            } else {
                Type st = visit(supertype(t));
                List<Type> is = visit(interfaces(t), ignored);
                return st == supertype(t) && is == interfaces(t) ? t : makeIntersectionType(is.prepend(st));
            }
        }
        @Override
        public Type visitWildcardType(WildcardType t, Void ignored) {
            WildcardType t2 = (WildcardType) super.visitWildcardType(t, ignored);
            if (t2 != t && t.isExtendsBound() && t2.type.isExtendsBound()) {
                t2.type = wildUpperBound(t2.type);
            }
            return t2;
        }
        @Override
        public Type visitForAll(ForAll t, Void ignored) {
            if (Type.containsAny(to, t.tvars)) {
                List<Type> freevars = newInstances(t.tvars);
                t = new ForAll(freevars, Types.this.subst(t.qtype, t.tvars, freevars));
            }
            List<Type> tvars1 = substBounds(t.tvars, from, to);
            Type qtype1 = visit(t.qtype);
            return tvars1 == t.tvars && qtype1 == t.qtype ? t : tvars1 == t.tvars ? new ForAll(tvars1, qtype1) {
                    @Override
                    public boolean needsStripping() {
                        return true;
                    }
                } : new ForAll(tvars1, Types.this.subst(qtype1, t.tvars, tvars1)) {
                    @Override
                    public boolean needsStripping() {
                        return true;
                    }
                };
        }
    }
    public List<Type> substBounds(List<Type> tvars, List<Type> from, List<Type> to) {
        if (tvars.isEmpty()) 
            return tvars;
        ListBuffer<Type> newBoundsBuf = new ListBuffer<>();
        boolean changed = false;
        for (Type t : tvars) {
            TypeVar tv = (TypeVar) t;
            Type bound = subst(tv.getUpperBound(), from, to);
            if (bound != tv.getUpperBound()) 
                changed = true;
            newBoundsBuf.append(bound);
        }
        if (!changed) 
            return tvars;
        ListBuffer<Type> newTvars = new ListBuffer<>();
        for (Type t : tvars) {
            newTvars.append(new TypeVar(t.tsym, null, syms.botType, t.getMetadata()));
        }
        List<Type> newBounds = newBoundsBuf.toList();
        from = tvars;
        to = newTvars.toList();
        for (; !newBounds.isEmpty(); newBounds = newBounds.tail) {
            newBounds.head = subst(newBounds.head, from, to);
        }
        newBounds = newBoundsBuf.toList();
        for (Type t : newTvars.toList()) {
            ((TypeVar) t).setUpperBound(newBounds.head);
            newBounds = newBounds.tail;
        }
        return newTvars.toList();
    }
    public TypeVar substBound(TypeVar t, List<Type> from, List<Type> to) {
        Type bound1 = subst(t.getUpperBound(), from, to);
        if (bound1 == t.getUpperBound()) 
            return t; else {
            TypeVar tv = new TypeVar(t.tsym, null, syms.botType, t.getMetadata());
            tv.setUpperBound(subst(bound1, List.of(t), List.of(tv)));
            return tv;
        }
    }
    public boolean hasSameBounds(ForAll t, ForAll s) {
        List<Type> l1 = t.tvars;
        List<Type> l2 = s.tvars;
        while (l1.nonEmpty() && l2.nonEmpty() && isSameType(l1.head.getUpperBound(), subst(l2.head.getUpperBound(), s.tvars, t.tvars))) {
            l1 = l1.tail;
            l2 = l2.tail;
        }
        return l1.isEmpty() && l2.isEmpty();
    }
    public List<Type> newInstances(List<Type> tvars) {
        List<Type> tvars1 = tvars.map(newInstanceFun);
        for (List<Type> l = tvars1; l.nonEmpty(); l = l.tail) {
            TypeVar tv = (TypeVar) l.head;
            tv.setUpperBound(subst(tv.getUpperBound(), tvars, tvars1));
        }
        return tvars1;
    }
    private static final TypeMapping<Void> newInstanceFun = new TypeMapping<Void>() {
            @Override
            public TypeVar visitTypeVar(TypeVar t, Void _unused) {
                return new TypeVar(t.tsym, t.getUpperBound(), t.getLowerBound(), t.getMetadata());
            }
        };
    public Type createMethodTypeWithParameters(Type original, List<Type> newParams) {
        return original.accept(methodWithParameters, newParams);
    }
    private final MapVisitor<List<Type>> methodWithParameters = new MapVisitor<List<Type>>() {
            public Type visitType(Type t, List<Type> newParams) {
                throw new IllegalArgumentException("Not a method type: " + t);
            }
            public Type visitMethodType(MethodType t, List<Type> newParams) {
                return new MethodType(newParams, t.restype, t.thrown, t.tsym);
            }
            public Type visitForAll(ForAll t, List<Type> newParams) {
                return new ForAll(t.tvars, t.qtype.accept(this, newParams));
            }
        };
    public Type createMethodTypeWithThrown(Type original, List<Type> newThrown) {
        return original.accept(methodWithThrown, newThrown);
    }
    private final MapVisitor<List<Type>> methodWithThrown = new MapVisitor<List<Type>>() {
            public Type visitType(Type t, List<Type> newThrown) {
                throw new IllegalArgumentException("Not a method type: " + t);
            }
            public Type visitMethodType(MethodType t, List<Type> newThrown) {
                return new MethodType(t.argtypes, t.restype, newThrown, t.tsym);
            }
            public Type visitForAll(ForAll t, List<Type> newThrown) {
                return new ForAll(t.tvars, t.qtype.accept(this, newThrown));
            }
        };
    public Type createMethodTypeWithReturn(Type original, Type newReturn) {
        return original.accept(methodWithReturn, newReturn);
    }
    private final MapVisitor<Type> methodWithReturn = new MapVisitor<Type>() {
            public Type visitType(Type t, Type newReturn) {
                throw new IllegalArgumentException("Not a method type: " + t);
            }
            public Type visitMethodType(MethodType t, Type newReturn) {
                return new MethodType(t.argtypes, newReturn, t.thrown, t.tsym) {
                    @Override
                    public Type baseType() {
                        return t;
                    }
                };
            }
            public Type visitForAll(ForAll t, Type newReturn) {
                return new ForAll(t.tvars, t.qtype.accept(this, newReturn)) {
                    @Override
                    public Type baseType() {
                        return t;
                    }
                };
            }
        };
    public Type createErrorType(Type originalType) {
        return new ErrorType(originalType, syms.errSymbol);
    }
    public Type createErrorType(ClassSymbol c, Type originalType) {
        return new ErrorType(c, originalType);
    }
    public Type createErrorType(Name name, TypeSymbol container, Type originalType) {
        return new ErrorType(name, container, originalType);
    }
    public int rank(Type t) {
        switch (t.getTag()) {
            case CLASS:
                {
                    ClassType cls = (ClassType) t;
                    if (cls.rank_field < 0) {
                        Name fullname = cls.tsym.getQualifiedName();
                        if (fullname == names.java_lang_Object) 
                            cls.rank_field = 0; else {
                            int r = rank(supertype(cls));
                            for (List<Type> l = interfaces(cls); l.nonEmpty(); l = l.tail) {
                                if (rank(l.head) > r) 
                                    r = rank(l.head);
                            }
                            cls.rank_field = r + 1;
                        }
                    }
                    return cls.rank_field;
                }
            case TYPEVAR:
                {
                    TypeVar tvar = (TypeVar) t;
                    if (tvar.rank_field < 0) {
                        int r = rank(supertype(tvar));
                        for (List<Type> l = interfaces(tvar); l.nonEmpty(); l = l.tail) {
                            if (rank(l.head) > r) 
                                r = rank(l.head);
                        }
                        tvar.rank_field = r + 1;
                    }
                    return tvar.rank_field;
                }
            case ERROR:
            case NONE:
                return 0;
            default:
                throw new AssertionError();
        }
    }
    public String toString(Type t, Locale locale) {
        return Printer.createStandardPrinter(messages).visit(t, locale);
    }
    public String toString(Symbol t, Locale locale) {
        return Printer.createStandardPrinter(messages).visit(t, locale);
    }
    @Deprecated
    public String toString(Type t) {
        if (t.hasTag(FORALL)) {
            ForAll forAll = (ForAll) t;
            return typaramsString(forAll.tvars) + forAll.qtype;
        }
        return "" + t;
    }
    private String typaramsString(List<Type> tvars) {
        StringBuilder s = new StringBuilder();
        s.append('<');
        boolean first = true;
        for (Type t : tvars) {
            if (!first) 
                s.append(", ");
            first = false;
            appendTyparamString((TypeVar) t, s);
        }
        s.append('>');
        return s.toString();
    }
    private void appendTyparamString(TypeVar t, StringBuilder buf) {
        buf.append(t);
        if (t.getUpperBound() == null || t.getUpperBound().tsym.getQualifiedName() == names.java_lang_Object) 
            return;
        buf.append(" extends ");
        Type bound = t.getUpperBound();
        if (!bound.isCompound()) {
            buf.append(bound);
        } else if ((erasure(t).tsym.flags() & INTERFACE) == 0) {
            buf.append(supertype(t));
            for (Type intf : interfaces(t)) {
                buf.append('&');
                buf.append(intf);
            }
        } else {
            boolean first = true;
            for (Type intf : interfaces(t)) {
                if (!first) 
                    buf.append('&');
                first = false;
                buf.append(intf);
            }
        }
    }
    private Map<Type,List<Type>> closureCache = new HashMap<>();
    public List<Type> closure(Type t) {
        List<Type> cl = closureCache.get(t);
        if (cl == null) {
            Type st = supertype(t);
            cl = !t.isCompound() ? st.hasTag(CLASS) ? insert(closure(st), t) : st.hasTag(TYPEVAR) ? closure(st).prepend(t) : List.of(t) : closure(supertype(t));
            for (List<Type> l = interfaces(t); l.nonEmpty(); l = l.tail) 
                cl = union(cl, closure(l.head));
            closureCache.put(t, cl);
        }
        return cl;
    }
    public Collector<Type, ClosureHolder, List<Type>> closureCollector(boolean minClosure, BiPredicate<Type, Type> shouldSkip) {
        return Collector.of(() -> new ClosureHolder(minClosure, shouldSkip), ClosureHolder::add, ClosureHolder::merge, ClosureHolder::closure);
    }
    class ClosureHolder {
        List<Type> closure;
        final boolean minClosure;
        final BiPredicate<Type, Type> shouldSkip;
        ClosureHolder(boolean minClosure, BiPredicate<Type, Type> shouldSkip) {
            this.closure = List.nil();
            this.minClosure = minClosure;
            this.shouldSkip = shouldSkip;
        }
        void add(Type type) {
            closure = insert(closure, type, shouldSkip);
        }
        ClosureHolder merge(ClosureHolder other) {
            closure = union(closure, other.closure, shouldSkip);
            return this;
        }
        List<Type> closure() {
            return minClosure ? closureMin(closure) : closure;
        }
    }
    BiPredicate<Type, Type> basicClosureSkip = (t1, t2) -> t1.tsym == t2.tsym;
    public List<Type> insert(List<Type> cl, Type t, BiPredicate<Type, Type> shouldSkip) {
        return cl.isEmpty() ? cl.prepend(t) : shouldSkip.test(t, cl.head) ? cl : t.tsym.precedes(cl.head.tsym, this) ? cl.prepend(t) : insert(cl.tail, t, shouldSkip).prepend(cl.head);
    }
    public List<Type> insert(List<Type> cl, Type t) {
        return insert(cl, t, basicClosureSkip);
    }
    public List<Type> union(List<Type> cl1, List<Type> cl2, BiPredicate<Type, Type> shouldSkip) {
        return cl1.isEmpty() ? cl2 : cl2.isEmpty() ? cl1 : shouldSkip.test(cl1.head, cl2.head) ? union(cl1.tail, cl2.tail, shouldSkip).prepend(cl1.head) : cl2.head.tsym.precedes(cl1.head.tsym, this) ? union(cl1, cl2.tail, shouldSkip).prepend(cl2.head) : union(cl1.tail, cl2, shouldSkip).prepend(cl1.head);
    }
    public List<Type> union(List<Type> cl1, List<Type> cl2) {
        return union(cl1, cl2, basicClosureSkip);
    }
    public List<Type> intersect(List<Type> cl1, List<Type> cl2) {
        if (cl1 == cl2) 
            return cl1;
        if (cl1.isEmpty() || cl2.isEmpty()) 
            return List.nil();
        if (cl1.head.tsym.precedes(cl2.head.tsym, this)) 
            return intersect(cl1.tail, cl2);
        if (cl2.head.tsym.precedes(cl1.head.tsym, this)) 
            return intersect(cl1, cl2.tail);
        if (isSameType(cl1.head, cl2.head)) 
            return intersect(cl1.tail, cl2.tail).prepend(cl1.head);
        if (cl1.head.tsym == cl2.head.tsym && cl1.head.hasTag(CLASS) && cl2.head.hasTag(CLASS)) {
            if (cl1.head.isParameterized() && cl2.head.isParameterized()) {
                Type merge = merge(cl1.head, cl2.head);
                return intersect(cl1.tail, cl2.tail).prepend(merge);
            }
            if (cl1.head.isRaw() || cl2.head.isRaw()) 
                return intersect(cl1.tail, cl2.tail).prepend(erasure(cl1.head));
        }
        return intersect(cl1.tail, cl2.tail);
    }
    class TypePair {
        final Type t1;
        final Type t2;
        TypePair(Type t1, Type t2) {
            this.t1 = t1;
            this.t2 = t2;
        }
        @Override
            public int hashCode() {
            return 127 * Types.this.hashCode(t1) + Types.this.hashCode(t2);
        }
        @Override
            public boolean equals(Object obj) {
            return obj instanceof TypePair typePair && isSameType(t1, typePair.t1) && isSameType(t2, typePair.t2);
        }
    }
    Set<TypePair> mergeCache = new HashSet<>();
    private Type merge(Type c1, Type c2) {
        ClassType class1 = (ClassType) c1;
        List<Type> act1 = class1.getTypeArguments();
        List<Type> act2 = ((ClassType) c2).getTypeArguments();
        ListBuffer<Type> merged = new ListBuffer<>();
        List<Type> typarams = class1.tsym.type.getTypeArguments();
        while (act1.nonEmpty() && act2.nonEmpty() && typarams.nonEmpty()) {
            if (containsType(act1.head, act2.head)) {
                merged.append(act1.head);
            } else if (containsType(act2.head, act1.head)) {
                merged.append(act2.head);
            } else {
                TypePair pair = new TypePair(c1, c2);
                Type m;
                if (mergeCache.add(pair)) {
                    m = new WildcardType(lub(wildUpperBound(act1.head), wildUpperBound(act2.head)), BoundKind.EXTENDS, syms.boundClass);
                    mergeCache.remove(pair);
                } else {
                    m = new WildcardType(syms.objectType, BoundKind.UNBOUND, syms.boundClass);
                }
                merged.append(m.withTypeVar(typarams.head));
            }
            act1 = act1.tail;
            act2 = act2.tail;
            typarams = typarams.tail;
        }
        Assert.check(act1.isEmpty() && act2.isEmpty() && typarams.isEmpty());
        return new ClassType(class1.getEnclosingType(), merged.toList(), class1.tsym);
    }
    private Type compoundMin(List<Type> cl) {
        if (cl.isEmpty()) 
            return syms.objectType;
        List<Type> compound = closureMin(cl);
        return compound.isEmpty() ? null : compound.tail.isEmpty() ? compound.head : makeIntersectionType(compound);
    }
    private List<Type> closureMin(List<Type> cl) {
        ListBuffer<Type> classes = new ListBuffer<>();
        ListBuffer<Type> interfaces = new ListBuffer<>();
        Set<Type> toSkip = new HashSet<>();
        while (!cl.isEmpty()) {
            Type current = cl.head;
            boolean keep = !toSkip.contains(current);
            if (keep && current.hasTag(TYPEVAR)) {
                for (Type t : cl.tail) {
                    if (isSubtypeNoCapture(t, current)) {
                        keep = false;
                        break;
                    }
                }
            }
            if (keep) {
                if (current.isInterface()) 
                    interfaces.append(current); else 
                    classes.append(current);
                for (Type t : cl.tail) {
                    if (isSubtypeNoCapture(current, t)) 
                        toSkip.add(t);
                }
            }
            cl = cl.tail;
        }
        return classes.appendList(interfaces).toList();
    }
    public Type lub(List<Type> ts) {
        return lub(ts.toArray(new Type[ts.length()]));
    }
    public Type lub(Type... ts) {
        int UNKNOWN_BOUND = 0;
        int ARRAY_BOUND = 1;
        int CLASS_BOUND = 2;
        int[] kinds = new int[ts.length];
        int boundkind = UNKNOWN_BOUND;
        for (int i = 0; i < ts.length; i++) {
            Type t = ts[i];
            switch (t.getTag()) {
                case CLASS:
                    boundkind |= kinds[i] = CLASS_BOUND;
                    break;
                case ARRAY:
                    boundkind |= kinds[i] = ARRAY_BOUND;
                    break;
                case TYPEVAR:
                    do {
                        t = t.getUpperBound();
                    } while (t.hasTag(TYPEVAR));
                    if (t.hasTag(ARRAY)) {
                        boundkind |= kinds[i] = ARRAY_BOUND;
                    } else {
                        boundkind |= kinds[i] = CLASS_BOUND;
                    }
                    break;
                default:
                    kinds[i] = UNKNOWN_BOUND;
                    if (t.isPrimitive()) 
                        return syms.errType;
            }
        }
        switch (boundkind) {
            case 0:
                return syms.botType;
            case ARRAY_BOUND:
                Type[] elements = new Type[ts.length];
                for (int i = 0; i < ts.length; i++) {
                    Type elem = elements[i] = elemTypeFun.apply(ts[i]);
                    if (elem.isPrimitive()) {
                        Type first = ts[0];
                        for (int j = 1; j < ts.length; j++) {
                            if (!isSameType(first, ts[j])) {
                                return arraySuperType();
                            }
                        }
                        return first;
                    }
                }
                return new ArrayType(lub(elements), syms.arrayClass);
            case CLASS_BOUND:
                int startIdx = 0;
                for (int i = 0; i < ts.length; i++) {
                    Type t = ts[i];
                    if (t.hasTag(CLASS) || t.hasTag(TYPEVAR)) {
                        break;
                    } else {
                        startIdx++;
                    }
                }
                Assert.check(startIdx < ts.length);
                List<Type> cl = erasedSupertypes(ts[startIdx]);
                for (int i = startIdx + 1; i < ts.length; i++) {
                    Type t = ts[i];
                    if (t.hasTag(CLASS) || t.hasTag(TYPEVAR)) 
                        cl = intersect(cl, erasedSupertypes(t));
                }
                List<Type> mec = closureMin(cl);
                List<Type> candidates = List.nil();
                for (Type erasedSupertype : mec) {
                    List<Type> lci = List.of(asSuper(ts[startIdx], erasedSupertype.tsym));
                    for (int i = startIdx + 1; i < ts.length; i++) {
                        Type superType = asSuper(ts[i], erasedSupertype.tsym);
                        lci = intersect(lci, superType != null ? List.of(superType) : List.nil());
                    }
                    candidates = candidates.appendList(lci);
                }
                return compoundMin(candidates);
            default:
                List<Type> classes = List.of(arraySuperType());
                for (int i = 0; i < ts.length; i++) {
                    if (kinds[i] != ARRAY_BOUND) 
                        classes = classes.prepend(ts[i]);
                }
                return lub(classes);
        }
    }
    List<Type> erasedSupertypes(Type t) {
        ListBuffer<Type> buf = new ListBuffer<>();
        for (Type sup : closure(t)) {
            if (sup.hasTag(TYPEVAR)) {
                buf.append(sup);
            } else {
                buf.append(erasure(sup));
            }
        }
        return buf.toList();
    }
    private Type arraySuperType;
    private Type arraySuperType() {
        if (arraySuperType == null) {
            arraySuperType = makeIntersectionType(List.of(syms.serializableType, syms.cloneableType), true);
        }
        return arraySuperType;
    }
    public Type glb(List<Type> ts) {
        Type t1 = ts.head;
        for (Type t2 : ts.tail) {
            if (t1.isErroneous()) 
                return t1;
            t1 = glb(t1, t2);
        }
        return t1;
    }
    public Type glb(Type t, Type s) {
        if (s == null) 
            return t; else if (t.isPrimitive() || s.isPrimitive()) 
            return syms.errType; else if (isSubtypeNoCapture(t, s)) 
            return t; else if (isSubtypeNoCapture(s, t)) 
            return s;
        List<Type> closure = union(closure(t), closure(s));
        return glbFlattened(closure, t);
    }
    private Type glbFlattened(List<Type> flatBounds, Type errT) {
        List<Type> bounds = closureMin(flatBounds);
        if (bounds.isEmpty()) {
            return syms.objectType;
        } else if (bounds.tail.isEmpty()) {
            return bounds.head;
        } else {
            int classCount = 0;
            List<Type> cvars = List.nil();
            List<Type> lowers = List.nil();
            for (Type bound : bounds) {
                if (!bound.isInterface()) {
                    classCount++;
                    Type lower = cvarLowerBound(bound);
                    if (bound != lower && !lower.hasTag(BOT)) {
                        cvars = cvars.append(bound);
                        lowers = lowers.append(lower);
                    }
                }
            }
            if (classCount > 1) {
                return lowers.isEmpty() ? createErrorType(errT) : glb(bounds.diff(cvars).appendList(lowers));
            }
        }
        return makeIntersectionType(bounds);
    }
    public int hashCode(Type t) {
        return hashCode(t, false);
    }
    public int hashCode(Type t, boolean strict) {
        return strict ? hashCodeStrictVisitor.visit(t) : hashCodeVisitor.visit(t);
    }
    private static final HashCodeVisitor hashCodeVisitor = new HashCodeVisitor();
    private static final HashCodeVisitor hashCodeStrictVisitor = new HashCodeVisitor() {
            @Override
            public Integer visitTypeVar(TypeVar t, Void ignored) {
                return System.identityHashCode(t);
            }
        };
    private static class HashCodeVisitor extends UnaryVisitor<Integer> {
        public Integer visitType(Type t, Void ignored) {
            return t.getTag().ordinal();
        }
        @Override
            public Integer visitClassType(ClassType t, Void ignored) {
            int result = visit(t.getEnclosingType()) * 127;
            result += t.tsym.flatName().hashCode();
            for (Type s : t.getTypeArguments()) {
                result *= 127;
                result += visit(s);
            }
            return result;
        }
        @Override
            public Integer visitMethodType(MethodType t, Void ignored) {
            int h = METHOD.ordinal();
            for (List<Type> thisargs = t.argtypes; thisargs.tail != null; thisargs = thisargs.tail) 
                h = (h << 5) + visit(thisargs.head);
            return (h << 5) + visit(t.restype);
        }
        @Override
            public Integer visitWildcardType(WildcardType t, Void ignored) {
            int result = t.kind.hashCode();
            if (t.type != null) {
                result *= 127;
                result += visit(t.type);
            }
            return result;
        }
        @Override
            public Integer visitArrayType(ArrayType t, Void ignored) {
            return visit(t.elemtype) + 12;
        }
        @Override
            public Integer visitTypeVar(TypeVar t, Void ignored) {
            return System.identityHashCode(t);
        }
        @Override
            public Integer visitUndetVar(UndetVar t, Void ignored) {
            return System.identityHashCode(t);
        }
        @Override
            public Integer visitErrorType(ErrorType t, Void ignored) {
            return 0;
        }
    }
    public boolean resultSubtype(Type t, Type s, Warner warner) {
        List<Type> tvars = t.getTypeArguments();
        List<Type> svars = s.getTypeArguments();
        return covariantReturnType(t.getReturnType(), subst(s.getReturnType(), svars, tvars), warner);
    }
    public boolean returnTypeSubstitutable(Type r1, Type r2) {
        return hasSameArgs(r1, r2) ? resultSubtype(r1, r2, noWarnings) : covariantReturnType(r1.getReturnType(), erasure(r2.getReturnType()), noWarnings);
    }
    public boolean returnTypeSubstitutable(Type r1, Type r2, Type r2res, Warner warner) {
        if (isSameType(r1.getReturnType(), r2res)) 
            return true;
        if (r1.getReturnType().isPrimitive() || r2res.isPrimitive()) 
            return false;
        if (hasSameArgs(r1, r2)) 
            return covariantReturnType(r1.getReturnType(), r2res, warner);
        if (isSubtypeUnchecked(r1.getReturnType(), r2res, warner)) 
            return true;
        if (!isSubtype(r1.getReturnType(), erasure(r2res))) 
            return false;
        warner.warn(LintCategory.UNCHECKED);
        return true;
    }
    public boolean covariantReturnType(Type t, Type s, Warner warner) {
        return isSameType(t, s) || !t.isPrimitive() && !s.isPrimitive() && isAssignable(t, s, warner);
    }
    public ClassSymbol boxedClass(Type t) {
        return syms.enterClass(syms.java_base, syms.boxedName[t.getTag().ordinal()]);
    }
    public Type boxedTypeOrType(Type t) {
        return t.isPrimitive() ? boxedClass(t).type : t;
    }
    public Type unboxedType(Type t) {
        if (t.hasTag(ERROR)) 
            return Type.noType;
        for (int i = 0; i < syms.boxedName.length; i++) {
            Name box = syms.boxedName[i];
            if (box != null && asSuper(t, syms.enterClass(syms.java_base, box)) != null) 
                return syms.typeOfTag[i];
        }
        return Type.noType;
    }
    public Type unboxedTypeOrType(Type t) {
        Type unboxedType = unboxedType(t);
        return unboxedType.hasTag(NONE) ? t : unboxedType;
    }
    public List<Type> capture(List<Type> ts) {
        List<Type> buf = List.nil();
        for (Type t : ts) {
            buf = buf.prepend(capture(t));
        }
        return buf.reverse();
    }
    public Type capture(Type t) {
        if (!t.hasTag(CLASS)) {
            return t;
        }
        if (t.getEnclosingType() != Type.noType) {
            Type capturedEncl = capture(t.getEnclosingType());
            if (capturedEncl != t.getEnclosingType()) {
                t = subst(memberType(capturedEncl, t.tsym), t.tsym.type.getTypeArguments(), t.getTypeArguments());
            }
        }
        ClassType cls = (ClassType) t;
        if (cls.isRaw() || !cls.isParameterized()) 
            return cls;
        List<Type> A = ((ClassType) cls.asElement().asType()).getTypeArguments();
        List<Type> T = cls.getTypeArguments();
        List<Type> S = freshTypeVariables(T);
        List<Type> currentA = A;
        List<Type> currentT = T;
        List<Type> currentS = S;
        boolean captured = false;
        while (!currentA.isEmpty() && !currentT.isEmpty() && !currentS.isEmpty()) {
            if (currentS.head != currentT.head) {
                captured = true;
                WildcardType Ti = (WildcardType) currentT.head;
                Type Ui = currentA.head.getUpperBound();
                CapturedType Si = (CapturedType) currentS.head;
                if (Ui == null) 
                    Ui = syms.objectType;
                switch (Ti.kind) {
                    case UNBOUND:
                        Si.setUpperBound(subst(Ui, A, S));
                        Si.lower = syms.botType;
                        break;
                    case EXTENDS:
                        Si.setUpperBound(glb(Ti.getExtendsBound(), subst(Ui, A, S)));
                        Si.lower = syms.botType;
                        break;
                    case SUPER:
                        Si.setUpperBound(subst(Ui, A, S));
                        Si.lower = Ti.getSuperBound();
                        break;
                }
                Type tmpBound = Si.getUpperBound().hasTag(UNDETVAR) ? ((UndetVar) Si.getUpperBound()).qtype : Si.getUpperBound();
                Type tmpLower = Si.lower.hasTag(UNDETVAR) ? ((UndetVar) Si.lower).qtype : Si.lower;
                if (!Si.getUpperBound().hasTag(ERROR) && !Si.lower.hasTag(ERROR) && isSameType(tmpBound, tmpLower)) {
                    currentS.head = Si.getUpperBound();
                }
            }
            currentA = currentA.tail;
            currentT = currentT.tail;
            currentS = currentS.tail;
        }
        return !currentA.isEmpty() || !currentT.isEmpty() || !currentS.isEmpty() ? erasure(t) : captured ? new ClassType(cls.getEnclosingType(), S, cls.tsym, cls.getMetadata()) : t;
    }
    public List<Type> freshTypeVariables(List<Type> types) {
        ListBuffer<Type> result = new ListBuffer<>();
        for (Type t : types) {
            if (t.hasTag(WILDCARD)) {
                Type bound = ((WildcardType) t).getExtendsBound();
                if (bound == null) 
                    bound = syms.objectType;
                result.append(new CapturedType(capturedName, syms.noSymbol, bound, syms.botType, (WildcardType) t));
            } else {
                result.append(t);
            }
        }
        return result.toList();
    }
    private boolean sideCast(Type from, Type to, Warner warn) {
        boolean reverse = false;
        Type target = to;
        if ((to.tsym.flags() & INTERFACE) == 0) {
            Assert.check((from.tsym.flags() & INTERFACE) != 0);
            reverse = true;
            to = from;
            from = target;
        }
        List<Type> commonSupers = superClosure(to, erasure(from));
        boolean giveWarning = commonSupers.isEmpty();
        while (commonSupers.nonEmpty()) {
            Type t1 = asSuper(from, commonSupers.head.tsym);
            Type t2 = commonSupers.head;
            if (disjointTypes(t1.getTypeArguments(), t2.getTypeArguments())) 
                return false;
            giveWarning = giveWarning || (reverse ? giveWarning(t2, t1) : giveWarning(t1, t2));
            commonSupers = commonSupers.tail;
        }
        if (giveWarning && !isReifiable(reverse ? from : to)) 
            warn.warn(LintCategory.UNCHECKED);
        return true;
    }
    private boolean sideCastFinal(Type from, Type to, Warner warn) {
        boolean reverse = false;
        Type target = to;
        if ((to.tsym.flags() & INTERFACE) == 0) {
            Assert.check((from.tsym.flags() & INTERFACE) != 0);
            reverse = true;
            to = from;
            from = target;
        }
        Assert.check((from.tsym.flags() & FINAL) != 0);
        Type t1 = asSuper(from, to.tsym);
        if (t1 == null) 
            return false;
        Type t2 = to;
        if (disjointTypes(t1.getTypeArguments(), t2.getTypeArguments())) 
            return false;
        if (!isReifiable(target) && (reverse ? giveWarning(t2, t1) : giveWarning(t1, t2))) 
            warn.warn(LintCategory.UNCHECKED);
        return true;
    }
    private boolean giveWarning(Type from, Type to) {
        List<Type> bounds = to.isCompound() ? directSupertypes(to) : List.of(to);
        for (Type b : bounds) {
            Type subFrom = asSub(from, b.tsym);
            if (b.isParameterized() && !(isUnbounded(b) || isSubtype(from, b) || subFrom != null && containsType(b.allparams(), subFrom.allparams()))) {
                return true;
            }
        }
        return false;
    }
    private List<Type> superClosure(Type t, Type s) {
        List<Type> cl = List.nil();
        for (List<Type> l = interfaces(t); l.nonEmpty(); l = l.tail) {
            cl = isSubtype(s, erasure(l.head)) ? insert(cl, l.head) : union(cl, superClosure(l.head, s));
        }
        return cl;
    }
    private boolean containsTypeEquivalent(Type t, Type s) {
        return isSameType(t, s) || containsType(t, s) && containsType(s, t);
    }
    public void adapt(Type source, Type target, ListBuffer<Type> from, ListBuffer<Type> to) throws AdaptFailure {
        new Adapter(from, to).adapt(source, target);
    }
    class Adapter extends SimpleVisitor<Void, Type> {
        ListBuffer<Type> from;
        ListBuffer<Type> to;
        Map<Symbol,Type> mapping;
        Adapter(ListBuffer<Type> from, ListBuffer<Type> to) {
            this.from = from;
            this.to = to;
            mapping = new HashMap<>();
        }
        public void adapt(Type source, Type target) throws AdaptFailure {
            visit(source, target);
            List<Type> fromList = from.toList();
            List<Type> toList = to.toList();
            while (!fromList.isEmpty()) {
                Type val = mapping.get(fromList.head.tsym);
                if (toList.head != val) 
                    toList.head = val;
                fromList = fromList.tail;
                toList = toList.tail;
            }
        }
        @Override
        public Void visitClassType(ClassType source, Type target) throws AdaptFailure {
            if (target.hasTag(CLASS)) 
                adaptRecursive(source.allparams(), target.allparams());
            return null;
        }
        @Override
        public Void visitArrayType(ArrayType source, Type target) throws AdaptFailure {
            if (target.hasTag(ARRAY)) 
                adaptRecursive(elemtype(source), elemtype(target));
            return null;
        }
        @Override
        public Void visitWildcardType(WildcardType source, Type target) throws AdaptFailure {
            if (source.isExtendsBound()) 
                adaptRecursive(wildUpperBound(source), wildUpperBound(target)); else if (source.isSuperBound()) 
                adaptRecursive(wildLowerBound(source), wildLowerBound(target));
            return null;
        }
        @Override
        public Void visitTypeVar(TypeVar source, Type target) throws AdaptFailure {
            Type val = mapping.get(source.tsym);
            if (val != null) {
                if (val.isSuperBound() && target.isSuperBound()) {
                    val = isSubtype(wildLowerBound(val), wildLowerBound(target)) ? target : val;
                } else if (val.isExtendsBound() && target.isExtendsBound()) {
                    val = isSubtype(wildUpperBound(val), wildUpperBound(target)) ? val : target;
                } else if (!isSameType(val, target)) {
                    throw new AdaptFailure();
                }
            } else {
                val = target;
                from.append(source);
                to.append(target);
            }
            mapping.put(source.tsym, val);
            return null;
        }
        @Override
        public Void visitType(Type source, Type target) {
            return null;
        }
        private Set<TypePair> cache = new HashSet<>();
        private void adaptRecursive(Type source, Type target) {
            TypePair pair = new TypePair(source, target);
            if (cache.add(pair)) {
                try {
                    visit(source, target);
                } finally {
                    cache.remove(pair);
                }
            }
        }
        private void adaptRecursive(List<Type> source, List<Type> target) {
            if (source.length() == target.length()) {
                while (source.nonEmpty()) {
                    adaptRecursive(source.head, target.head);
                    source = source.tail;
                    target = target.tail;
                }
            }
        }
    }
    public static class AdaptFailure extends RuntimeException {
        static final long serialVersionUID = -7490231548272701566L;
    }
    private void adaptSelf(Type t, ListBuffer<Type> from, ListBuffer<Type> to) {
        try {
            adapt(t.tsym.type, t, from, to);
        } catch (AdaptFailure ex) {
            throw new AssertionError(ex);
        }
    }
    private Type rewriteQuantifiers(Type t, boolean high, boolean rewriteTypeVars) {
        return new Rewriter(high, rewriteTypeVars).visit(t);
    }
    class Rewriter extends UnaryVisitor<Type> {
        boolean high;
        boolean rewriteTypeVars;
        Rewriter(boolean high, boolean rewriteTypeVars) {
            this.high = high;
            this.rewriteTypeVars = rewriteTypeVars;
        }
        @Override
        public Type visitClassType(ClassType t, Void s) {
            ListBuffer<Type> rewritten = new ListBuffer<>();
            boolean changed = false;
            for (Type arg : t.allparams()) {
                Type bound = visit(arg);
                if (arg != bound) {
                    changed = true;
                }
                rewritten.append(bound);
            }
            return changed ? subst(t.tsym.type, t.tsym.type.allparams(), rewritten.toList()) : t;
        }
        public Type visitType(Type t, Void s) {
            return t;
        }
        @Override
        public Type visitCapturedType(CapturedType t, Void s) {
            Type w_bound = t.wildcard.type;
            return rewriteAsWildcardType(visit(w_bound.contains(t) ? erasure(w_bound) : visit(w_bound)), t.wildcard.bound, t.wildcard.kind);
        }
        @Override
        public Type visitTypeVar(TypeVar t, Void s) {
            return rewriteTypeVars ? rewriteAsWildcardType(t.getUpperBound().contains(t) ? erasure(t.getUpperBound()) : visit(t.getUpperBound()), t, EXTENDS) : t;
        }
        @Override
        public Type visitWildcardType(WildcardType t, Void s) {
            Type bound2 = visit(t.type);
            return t.type == bound2 ? t : rewriteAsWildcardType(bound2, t.bound, t.kind);
        }
        private Type rewriteAsWildcardType(Type bound, TypeVar formal, BoundKind bk) {
            switch (bk) {
                case EXTENDS:
                    return high ? makeExtendsWildcard(B(bound), formal) : makeExtendsWildcard(syms.objectType, formal);
                case SUPER:
                    return high ? makeSuperWildcard(syms.botType, formal) : makeSuperWildcard(B(bound), formal);
                case UNBOUND:
                    return makeExtendsWildcard(syms.objectType, formal);
                default:
                    Assert.error("Invalid bound kind " + bk);
                    return null;
            }
        }
        Type B(Type t) {
            while (t.hasTag(WILDCARD)) {
                t = ((WildcardType) t).getExtendsBound();
                if (t == null) {
                    t = syms.objectType;
                }
            }
            return t;
        }
    }
    private WildcardType makeExtendsWildcard(Type bound, TypeVar formal) {
        return bound == syms.objectType ? new WildcardType(syms.objectType, BoundKind.UNBOUND, syms.boundClass, formal) : new WildcardType(bound, BoundKind.EXTENDS, syms.boundClass, formal);
    }
    private WildcardType makeSuperWildcard(Type bound, TypeVar formal) {
        return bound.hasTag(BOT) ? new WildcardType(syms.objectType, BoundKind.UNBOUND, syms.boundClass, formal) : new WildcardType(bound, BoundKind.SUPER, syms.boundClass, formal);
    }
    public static class UniqueType {
        public final Type type;
        final Types types;
        public UniqueType(Type type, Types types) {
            this.type = type;
            this.types = types;
        }
        public int hashCode() {
            return types.hashCode(type);
        }
        public boolean equals(Object obj) {
            return obj instanceof UniqueType uniqueType && types.isSameType(type, uniqueType.type);
        }
        public String toString() {
            return type.toString();
        }
    }
    public abstract static class DefaultTypeVisitor<R,S> implements Type.Visitor<R,S> {
        public final R visit(Type t, S s) {
            return t.accept(this, s);
        }
        public R visitClassType(ClassType t, S s) {
            return visitType(t, s);
        }
        public R visitWildcardType(WildcardType t, S s) {
            return visitType(t, s);
        }
        public R visitArrayType(ArrayType t, S s) {
            return visitType(t, s);
        }
        public R visitMethodType(MethodType t, S s) {
            return visitType(t, s);
        }
        public R visitPackageType(PackageType t, S s) {
            return visitType(t, s);
        }
        public R visitModuleType(ModuleType t, S s) {
            return visitType(t, s);
        }
        public R visitTypeVar(TypeVar t, S s) {
            return visitType(t, s);
        }
        public R visitCapturedType(CapturedType t, S s) {
            return visitType(t, s);
        }
        public R visitForAll(ForAll t, S s) {
            return visitType(t, s);
        }
        public R visitUndetVar(UndetVar t, S s) {
            return visitType(t, s);
        }
        public R visitErrorType(ErrorType t, S s) {
            return visitType(t, s);
        }
    }
    public abstract static class DefaultSymbolVisitor<R,S> implements Symbol.Visitor<R,S> {
        public final R visit(Symbol s, S arg) {
            return s.accept(this, arg);
        }
        public R visitClassSymbol(ClassSymbol s, S arg) {
            return visitSymbol(s, arg);
        }
        public R visitMethodSymbol(MethodSymbol s, S arg) {
            return visitSymbol(s, arg);
        }
        public R visitOperatorSymbol(OperatorSymbol s, S arg) {
            return visitSymbol(s, arg);
        }
        public R visitPackageSymbol(PackageSymbol s, S arg) {
            return visitSymbol(s, arg);
        }
        public R visitTypeSymbol(TypeSymbol s, S arg) {
            return visitSymbol(s, arg);
        }
        public R visitVarSymbol(VarSymbol s, S arg) {
            return visitSymbol(s, arg);
        }
    }
    public abstract static class SimpleVisitor<R,S> extends DefaultTypeVisitor<R,S> {
        @Override
        public R visitCapturedType(CapturedType t, S s) {
            return visitTypeVar(t, s);
        }
        @Override
        public R visitForAll(ForAll t, S s) {
            return visit(t.qtype, s);
        }
        @Override
        public R visitUndetVar(UndetVar t, S s) {
            return visit(t.qtype, s);
        }
    }
    public abstract static class TypeRelation extends SimpleVisitor<Boolean,Type> {
    }
    public abstract static class UnaryVisitor<R> extends SimpleVisitor<R,Void> {
        public final R visit(Type t) {
            return t.accept(this, null);
        }
    }
    public static class MapVisitor<S> extends DefaultTypeVisitor<Type,S> {
        public final Type visit(Type t) {
            return t.accept(this, null);
        }
        public Type visitType(Type t, S s) {
            return t;
        }
    }
    public static class TypeMapping<S> extends MapVisitor<S> implements Function<Type, Type> {
        @Override
        public Type apply(Type type) {
            return visit(type);
        }
        List<Type> visit(List<Type> ts, S s) {
            return ts.map((t) -> visit(t, s));
        }
        @Override
        public Type visitCapturedType(CapturedType t, S s) {
            return visitTypeVar(t, s);
        }
    }
    public RetentionPolicy getRetention(Attribute.Compound a) {
        return getRetention(a.type.tsym);
    }
    public RetentionPolicy getRetention(TypeSymbol sym) {
        RetentionPolicy vis = RetentionPolicy.CLASS;
        Attribute.Compound c = sym.attribute(syms.retentionType.tsym);
        if (c != null) {
            Attribute value = c.member(names.value);
            if (value != null && value instanceof Attribute.Enum attributeEnum) {
                Name levelName = attributeEnum.value.name;
                if (levelName == names.SOURCE) 
                    vis = RetentionPolicy.SOURCE; else if (levelName == names.CLASS) 
                    vis = RetentionPolicy.CLASS; else if (levelName == names.RUNTIME) 
                    vis = RetentionPolicy.RUNTIME; else ;
            }
        }
        return vis;
    }
    public abstract static class SignatureGenerator {
        public static class InvalidSignatureException extends RuntimeException {
            private static final long serialVersionUID = 0;
            private final transient Type type;
            InvalidSignatureException(Type type) {
                this.type = type;
            }
            public Type type() {
                return type;
            }
            @Override
            public Throwable fillInStackTrace() {
                return this;
            }
        }
        private final Types types;
        protected abstract void append(char ch);
        protected abstract void append(byte[] ba);
        protected abstract void append(Name name);
        protected void classReference(ClassSymbol c) {}
        protected SignatureGenerator(Types types) {
            this.types = types;
        }
        protected void reportIllegalSignature(Type t) {
            throw new InvalidSignatureException(t);
        }
        public void assembleSig(Type type) {
            switch (type.getTag()) {
                case BYTE:
                    append('B');
                    break;
                case SHORT:
                    append('S');
                    break;
                case CHAR:
                    append('C');
                    break;
                case INT:
                    append('I');
                    break;
                case LONG:
                    append('J');
                    break;
                case FLOAT:
                    append('F');
                    break;
                case DOUBLE:
                    append('D');
                    break;
                case BOOLEAN:
                    append('Z');
                    break;
                case VOID:
                    append('V');
                    break;
                case CLASS:
                    if (type.isCompound()) {
                        reportIllegalSignature(type);
                    }
                    append('L');
                    assembleClassSig(type);
                    append(';');
                    break;
                case ARRAY:
                    ArrayType at = (ArrayType) type;
                    append('[');
                    assembleSig(at.elemtype);
                    break;
                case METHOD:
                    MethodType mt = (MethodType) type;
                    append('(');
                    assembleSig(mt.argtypes);
                    append(')');
                    assembleSig(mt.restype);
                    if (hasTypeVar(mt.thrown)) {
                        for (List<Type> l = mt.thrown; l.nonEmpty(); l = l.tail) {
                            append('^');
                            assembleSig(l.head);
                        }
                    }
                    break;
                case WILDCARD:
                    {
                        Type.WildcardType ta = (Type.WildcardType) type;
                        switch (ta.kind) {
                            case SUPER:
                                append('-');
                                assembleSig(ta.type);
                                break;
                            case EXTENDS:
                                append('+');
                                assembleSig(ta.type);
                                break;
                            case UNBOUND:
                                append('*');
                                break;
                            default:
                                throw new AssertionError(ta.kind);
                        }
                        break;
                    }
                case TYPEVAR:
                    if (((TypeVar) type).isCaptured()) {
                        reportIllegalSignature(type);
                    }
                    append('T');
                    append(type.tsym.name);
                    append(';');
                    break;
                case FORALL:
                    Type.ForAll ft = (Type.ForAll) type;
                    assembleParamsSig(ft.tvars);
                    assembleSig(ft.qtype);
                    break;
                default:
                    throw new AssertionError("typeSig " + type.getTag());
            }
        }
        public boolean hasTypeVar(List<Type> l) {
            while (l.nonEmpty()) {
                if (l.head.hasTag(TypeTag.TYPEVAR)) {
                    return true;
                }
                l = l.tail;
            }
            return false;
        }
        public void assembleClassSig(Type type) {
            ClassType ct = (ClassType) type;
            ClassSymbol c = (ClassSymbol) ct.tsym;
            classReference(c);
            Type outer = ct.getEnclosingType();
            if (outer.allparams().nonEmpty()) {
                boolean rawOuter = c.owner.kind == MTH || c.name == types.names.empty;
                assembleClassSig(rawOuter ? types.erasure(outer) : outer);
                append(rawOuter ? '$' : '.');
                Assert.check(c.flatname.startsWith(c.owner.enclClass().flatname));
                append(rawOuter ? c.flatname.subName(c.owner.enclClass().flatname.getByteLength() + 1, c.flatname.getByteLength()) : c.name);
            } else {
                append(externalize(c.flatname));
            }
            if (ct.getTypeArguments().nonEmpty()) {
                append('<');
                assembleSig(ct.getTypeArguments());
                append('>');
            }
        }
        public void assembleParamsSig(List<Type> typarams) {
            append('<');
            for (List<Type> ts = typarams; ts.nonEmpty(); ts = ts.tail) {
                Type.TypeVar tvar = (Type.TypeVar) ts.head;
                append(tvar.tsym.name);
                List<Type> bounds = types.getBounds(tvar);
                if ((bounds.head.tsym.flags() & INTERFACE) != 0) {
                    append(':');
                }
                for (List<Type> l = bounds; l.nonEmpty(); l = l.tail) {
                    append(':');
                    assembleSig(l.head);
                }
            }
            append('>');
        }
        public void assembleSig(List<Type> types) {
            for (List<Type> ts = types; ts.nonEmpty(); ts = ts.tail) {
                assembleSig(ts.head);
            }
        }
    }
    public Type constantType(LoadableConstant c) {
        switch (c.poolTag()) {
            case ClassFile.CONSTANT_Class:
                return syms.classType;
            case ClassFile.CONSTANT_String:
                return syms.stringType;
            case ClassFile.CONSTANT_Integer:
                return syms.intType;
            case ClassFile.CONSTANT_Float:
                return syms.floatType;
            case ClassFile.CONSTANT_Long:
                return syms.longType;
            case ClassFile.CONSTANT_Double:
                return syms.doubleType;
            case ClassFile.CONSTANT_MethodHandle:
                return syms.methodHandleType;
            case ClassFile.CONSTANT_MethodType:
                return syms.methodTypeType;
            case ClassFile.CONSTANT_Dynamic:
                return ((DynamicVarSymbol) c).type;
            default:
                throw new AssertionError("Not a loadable constant: " + c.poolTag());
        }
    }
    public void newRound() {
        descCache._map.clear();
        isDerivedRawCache.clear();
        implCache._map.clear();
        membersCache._map.clear();
        closureCache.clear();
    }
}
