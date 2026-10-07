import java.util.stream.Collectors;
import java.util.List;

public class LocalRecords {
    public interface Merchant {
    }
    public static double computeSales(Merchant merchant, int month) {
        return month;
    }
    List<Merchant> findTopMerchants(List<Merchant> merchants, int month) {
        record MerchantSales(Merchant merchant, double sales) {}

                return merchants.stream()
                    .map(merchant -> new MerchantSales(merchant, computeSales(merchant, month)))
                    .sorted((m1, m2) -> Double.compare(m2.sales(), m1.sales()))
                    .map(MerchantSales::merchant)
                    .collect(Collectors.toList());
    }
    void methodWithLocalRecordAndModifiers() {
        record MyRecord1(String a) {}
                final static record MyRecord2(String a) {}
                @Deprecated record MyRecord3(String a) {}
                final @Deprecated static record MyRecord4(String a) {}
    }
    void methodWithLocalClass() {
        class MyLocalClass {}
    }
    void methodWithLocalVarsNamedSealed() {
        System.out.println(-1);
    }
}
