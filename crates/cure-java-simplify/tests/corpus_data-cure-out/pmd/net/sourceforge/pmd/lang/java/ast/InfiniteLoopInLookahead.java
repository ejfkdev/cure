import java.util.*;

public class InfiniteLoopInLookahead {
    public void exam1(List resList) {
        resList.forEach((a) -> {
            resList.forEach((b) -> {
                resList.forEach((c) -> {
                    resList.forEach((d) -> {
                        resList.forEach((e) -> {
                            resList.forEach((f) -> {
                                resList.forEach((g) -> {
                                    resList.forEach((h) -> {
                                        resList;
                                    });
                                });
                            });
                        });
                    });
                });
            });
        });
    }
}
