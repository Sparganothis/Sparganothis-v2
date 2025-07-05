import logging

logging.basicConfig(level=logging.INFO)


def migrate():
    logging.info("STARTED MIGRATING DATABASE.....")
    db_host = "localhost"
    db_user = "sparganothis"
    db_password = "sparganothis"
    db_name = "sparganothis"
    migrations_home = "migrations"

    from clickhouse_migrations.clickhouse_cluster import ClickhouseCluster

    cluster = ClickhouseCluster(db_host, db_user, db_password)
    cluster.migrate(db_name, migrations_home, cluster_name=None,create_db_if_no_exists=True, multi_statement=True)

    logging.info("DATABASE MIGRATION FINISHED OK.")
if __name__ == '__main__':
    migrate()