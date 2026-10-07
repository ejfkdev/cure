import java.sql.Driver;
import java.sql.*;
import static java.sql.Types.INTEGER;
import static java.sql.Types.*;
import module java.base;

module java.sql {
    requires transitive java.logging;
    requires transitive java.transaction.xa;
    requires transitive java.xml;

    exports java.sql;
    exports javax.sql;

    uses Driver;
}
