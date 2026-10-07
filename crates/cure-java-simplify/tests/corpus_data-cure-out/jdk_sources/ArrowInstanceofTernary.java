package sun.security.util;

import java.io.IOException;
import java.security.*;
import java.security.interfaces.EdECKey;
import java.security.interfaces.EdECPrivateKey;
import java.security.interfaces.RSAKey;
import java.security.spec.*;
import java.util.Locale;
import sun.security.rsa.RSAUtil;
import jdk.internal.access.SharedSecrets;
import sun.security.x509.AlgorithmId;

public class SignatureUtil {
    private static String checkName(String algName) {
        if (algName.indexOf(".") == -1) {
            return algName;
        } else {
            if (algName.startsWith("OID.")) {
                algName = algName.substring(4);
            }
            KnownOIDs ko = KnownOIDs.findMatch(algName);
            return ko != null ? ko.stdName() : algName;
        }
    }
    private static AlgorithmParameters createAlgorithmParameters(String algName, byte[] paramBytes) throws ProviderException {
        try {
            algName = checkName(algName);
            AlgorithmParameters result = AlgorithmParameters.getInstance(algName);
            result.init(paramBytes);
            return result;
        } catch (NoSuchAlgorithmException | IOException e) {
            throw new ProviderException(e);
        }
    }
    public static AlgorithmParameterSpec getParamSpec(String sigName, AlgorithmParameters params) throws ProviderException {
        AlgorithmParameterSpec paramSpec = null;
        if (params != null) {
            sigName = checkName(sigName).toUpperCase(Locale.ENGLISH);
            if (params.getAlgorithm().indexOf(".") != -1) {
                try {
                    params = createAlgorithmParameters(sigName, params.getEncoded());
                } catch (IOException e) {
                    throw new ProviderException(e);
                }
            }
            if (sigName.indexOf("RSA") != -1) {
                paramSpec = RSAUtil.getParamSpec(params);
            } else if (sigName.indexOf("ECDSA") != -1) {
                try {
                    paramSpec = params.getParameterSpec(ECParameterSpec.class);
                } catch (Exception e) {
                    throw new ProviderException("Error handling EC parameters", e);
                }
            } else {
                throw new ProviderException("Unrecognized algorithm for signature parameters " + sigName);
            }
        }
        return paramSpec;
    }
    public static AlgorithmParameterSpec getParamSpec(String sigName, byte[] paramBytes) throws ProviderException {
        AlgorithmParameterSpec paramSpec = null;
        if (paramBytes != null) {
            sigName = checkName(sigName).toUpperCase(Locale.ENGLISH);
            if (sigName.indexOf("RSA") != -1) {
                AlgorithmParameters params = createAlgorithmParameters(sigName, paramBytes);
                paramSpec = RSAUtil.getParamSpec(params);
            } else if (sigName.indexOf("ECDSA") != -1) {
                try {
                    Provider p = Signature.getInstance(sigName).getProvider();
                    paramSpec = ECUtil.getECParameterSpec(p, paramBytes);
                } catch (Exception e) {
                    throw new ProviderException("Error handling EC parameters", e);
                }
                if (paramSpec == null) {
                    throw new ProviderException("Error handling EC parameters");
                }
            } else {
                throw new ProviderException("Unrecognized algorithm for signature parameters " + sigName);
            }
        }
        return paramSpec;
    }
    public static void initVerifyWithParam(Signature s, PublicKey key, AlgorithmParameterSpec params) throws ProviderException, InvalidAlgorithmParameterException, InvalidKeyException {
        SharedSecrets.getJavaSecuritySignatureAccess().initVerify(s, key, params);
    }
    public static void initVerifyWithParam(Signature s, java.security.cert.Certificate cert, AlgorithmParameterSpec params) throws ProviderException, InvalidAlgorithmParameterException, InvalidKeyException {
        SharedSecrets.getJavaSecuritySignatureAccess().initVerify(s, cert, params);
    }
    public static void initSignWithParam(Signature s, PrivateKey key, AlgorithmParameterSpec params, SecureRandom sr) throws ProviderException, InvalidAlgorithmParameterException, InvalidKeyException {
        SharedSecrets.getJavaSecuritySignatureAccess().initSign(s, key, params, sr);
    }
    public static class EdDSADigestAlgHolder {
        public final static AlgorithmId sha512;
        public final static AlgorithmId shake256;
        public final static AlgorithmId shake256$512;
        static {
            try {
                sha512 = new AlgorithmId(ObjectIdentifier.of(KnownOIDs.SHA_512));
                shake256 = new AlgorithmId(ObjectIdentifier.of(KnownOIDs.SHAKE256));
                shake256$512 = new AlgorithmId(ObjectIdentifier.of(KnownOIDs.SHAKE256_LEN), new DerValue((byte) 2, new byte[] {2, 0}));
            } catch (IOException e) {
                throw new AssertionError("Should not happen", e);
            }
        }
    }
    public static AlgorithmId getDigestAlgInPkcs7SignerInfo(Signature signer, String sigalg, PrivateKey privateKey, boolean directsign) throws NoSuchAlgorithmException {
        AlgorithmId digAlgID;
        String kAlg = privateKey.getAlgorithm();
        if (privateKey instanceof EdECPrivateKey || kAlg.equalsIgnoreCase("Ed25519") || kAlg.equalsIgnoreCase("Ed448")) {
            if (privateKey instanceof EdECPrivateKey) {
                kAlg = ((EdECPrivateKey) privateKey).getParams().getName();
            }
            switch (kAlg.toUpperCase(Locale.ENGLISH)) {
                case "ED25519":
                    digAlgID = EdDSADigestAlgHolder.sha512;
                    break;
                case "ED448":
                    digAlgID = EdDSADigestAlgHolder.shake256;
                    break;
                default:
                    throw new AssertionError("Unknown curve name: " + kAlg);
            }
        } else {
            if (sigalg.equalsIgnoreCase("RSASSA-PSS")) {
                try {
                    digAlgID = AlgorithmId.get(signer.getParameters().getParameterSpec(PSSParameterSpec.class).getDigestAlgorithm());
                } catch (InvalidParameterSpecException e) {
                    throw new AssertionError("Should not happen", e);
                }
            } else {
                digAlgID = AlgorithmId.get(extractDigestAlgFromDwithE(sigalg));
            }
        }
        return digAlgID;
    }
    public static String extractDigestAlgFromDwithE(String signatureAlgorithm) {
        signatureAlgorithm = signatureAlgorithm.toUpperCase(Locale.ENGLISH);
        int with = signatureAlgorithm.indexOf("WITH");
        if (with > 0) {
            return signatureAlgorithm.substring(0, with);
        } else {
            throw new IllegalArgumentException("Unknown algorithm: " + signatureAlgorithm);
        }
    }
    public static String extractKeyAlgFromDwithE(String signatureAlgorithm) {
        signatureAlgorithm = signatureAlgorithm.toUpperCase(Locale.ENGLISH);
        int with = signatureAlgorithm.indexOf("WITH");
        String keyAlgorithm = null;
        if (with > 0) {
            int and = signatureAlgorithm.indexOf("AND", with + 4);
            keyAlgorithm = and > 0 ? signatureAlgorithm.substring(with + 4, and) : signatureAlgorithm.substring(with + 4);
            if (keyAlgorithm.equalsIgnoreCase("ECDSA")) {
                keyAlgorithm = "EC";
            }
        }
        return keyAlgorithm;
    }
    public static AlgorithmParameterSpec getDefaultParamSpec(String sigAlg, Key k) {
        sigAlg = checkName(sigAlg);
        if (sigAlg.equalsIgnoreCase("RSASSA-PSS")) {
            if (k instanceof RSAKey) {
                AlgorithmParameterSpec spec = ((RSAKey) k).getParams();
                if (spec instanceof PSSParameterSpec) {
                    return spec;
                }
            }
            switch (ifcFfcStrength(KeyUtil.getKeySize(k))) {
                case "SHA256":
                    return PSSParamsHolder.PSS_256_SPEC;
                case "SHA384":
                    return PSSParamsHolder.PSS_384_SPEC;
                case "SHA512":
                    return PSSParamsHolder.PSS_512_SPEC;
                default:
                    throw new AssertionError("Should not happen");
            }
        } else {
            return null;
        }
    }
    public static Signature fromKey(String sigAlg, Key key, String provider) throws NoSuchAlgorithmException, NoSuchProviderException, InvalidKeyException {
        return autoInitInternal(sigAlg, key, provider == null || provider.isEmpty() ? Signature.getInstance(sigAlg) : Signature.getInstance(sigAlg, provider));
    }
    public static Signature fromKey(String sigAlg, Key key, Provider provider) throws NoSuchAlgorithmException, InvalidKeyException {
        return autoInitInternal(sigAlg, key, provider == null ? Signature.getInstance(sigAlg) : Signature.getInstance(sigAlg, provider));
    }
    private static Signature autoInitInternal(String alg, Key key, Signature s) throws InvalidKeyException {
        AlgorithmParameterSpec params = SignatureUtil.getDefaultParamSpec(alg, key);
        try {
            if (key instanceof PrivateKey) {
                SignatureUtil.initSignWithParam(s, (PrivateKey) key, params, null);
            } else {
                SignatureUtil.initVerifyWithParam(s, (PublicKey) key, params);
            }
        } catch (InvalidAlgorithmParameterException e) {
            throw new AssertionError("Should not happen", e);
        }
        return s;
    }
    public static AlgorithmId fromSignature(Signature sigEngine, PrivateKey key) throws SignatureException {
        try {
            if (key instanceof EdECKey) {
                return AlgorithmId.get(((EdECKey) key).getParams().getName());
            }
            AlgorithmParameters params = null;
            try {
                params = sigEngine.getParameters();
            } catch (UnsupportedOperationException e) {}
            if (params != null) {
                return AlgorithmId.get(sigEngine.getParameters());
            } else {
                String sigAlg = sigEngine.getAlgorithm();
                if (sigAlg.equalsIgnoreCase("EdDSA")) {
                    sigAlg = key.getAlgorithm();
                }
                return AlgorithmId.get(sigAlg);
            }
        } catch (NoSuchAlgorithmException e) {
            throw new SignatureException("Cannot derive AlgorithmIdentifier", e);
        }
    }
    public static void checkKeyAndSigAlgMatch(PrivateKey key, String sAlg) {
        String kAlg = key.getAlgorithm().toUpperCase(Locale.ENGLISH);
        sAlg = checkName(sAlg).toUpperCase(Locale.ENGLISH);
        switch (sAlg) {
            case "RSASSA-PSS" -> {
                if (!kAlg.equals("RSASSA-PSS") && !kAlg.equals("RSA")) {
                    throw new IllegalArgumentException("key algorithm not compatible with signature algorithm");
                }
            }
            case "EDDSA" -> {
                if (!kAlg.equals("EDDSA") && !kAlg.equals("ED448") && !kAlg.equals("ED25519")) {
                    throw new IllegalArgumentException("key algorithm not compatible with signature algorithm");
                }
            }
            case "ED25519", "ED448" -> {
                if (key instanceof EdECKey) {
                    String groupName = ((EdECKey) key).getParams().getName().toUpperCase(Locale.US);
                    if (!sAlg.equals(groupName)) {
                        throw new IllegalArgumentException("key algorithm not compatible with signature algorithm");
                    }
                } else {
                    if (!kAlg.equals("EDDSA") && !kAlg.equals(sAlg)) {
                        throw new IllegalArgumentException("key algorithm not compatible with signature algorithm");
                    }
                }
            }
            default -> {
                if (sAlg.contains("WITH")) {
                    if (sAlg.endsWith("WITHRSA") && !kAlg.equals("RSA") || sAlg.endsWith("WITHECDSA") && !kAlg.equals("EC") || sAlg.endsWith("WITHDSA") && !kAlg.equals("DSA")) {
                        throw new IllegalArgumentException("key algorithm not compatible with signature algorithm");
                    }
                }
            }
        }
    }
    public static String getDefaultSigAlgForKey(PrivateKey k) {
        String kAlg = k.getAlgorithm();
        return switch (kAlg.toUpperCase(Locale.ENGLISH)) {
            case "DSA", "RSA" -> ifcFfcStrength(KeyUtil.getKeySize(k)) + "with" + kAlg;
            case "EC" -> ecStrength(KeyUtil.getKeySize(k)) + "withECDSA";
            case "EDDSA" -> k instanceof EdECPrivateKey ? ((EdECPrivateKey) k).getParams().getName() : kAlg;
            case "RSASSA-PSS", "ED25519", "ED448" -> kAlg;
            default -> null;
        };
    }
    private static class PSSParamsHolder {
        final static PSSParameterSpec PSS_256_SPEC = new PSSParameterSpec("SHA-256", "MGF1", MGF1ParameterSpec.SHA256, 32, PSSParameterSpec.TRAILER_FIELD_BC);
        final static PSSParameterSpec PSS_384_SPEC = new PSSParameterSpec("SHA-384", "MGF1", MGF1ParameterSpec.SHA384, 48, PSSParameterSpec.TRAILER_FIELD_BC);
        final static PSSParameterSpec PSS_512_SPEC = new PSSParameterSpec("SHA-512", "MGF1", MGF1ParameterSpec.SHA512, 64, PSSParameterSpec.TRAILER_FIELD_BC);
    }
    private static String ecStrength(int bitLength) {
        return bitLength >= 512 ? "SHA512" : bitLength >= 384 ? "SHA384" : "SHA256";
    }
    private static String ifcFfcStrength(int bitLength) {
        return bitLength > 7680 ? "SHA512" : bitLength > 3072 ? "SHA384" : "SHA256";
    }
}
