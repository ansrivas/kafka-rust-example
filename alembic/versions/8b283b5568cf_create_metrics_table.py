"""create metrics table

Revision ID: 8b283b5568cf
Revises:
Create Date: 2019-12-26 20:04:47.614426

"""
from alembic import op
import sqlalchemy as sa
from sqlalchemy.dialects import postgresql

# revision identifiers, used by Alembic.
revision = "8b283b5568cf"
down_revision = None
branch_labels = None
depends_on = None


def upgrade():

    op.execute("CREATE EXTENSION IF NOT EXISTS timescaledb CASCADE;")
    op.create_table(
        "metrics",
        sa.Column("timestamp", postgresql.TIMESTAMP(precision=6, timezone=True), primary_key=True),
        sa.Column("name", sa.String(50), nullable=False, primary_key=True),
        sa.Column("value", sa.FLOAT),
    )
    op.execute("CREATE INDEX ON metrics (name, timestamp DESC);")
    op.execute("SELECT create_hypertable('metrics', 'timestamp');")


def downgrade():
    op.execute('DROP EXTENSION "timescaledb" CASCADE;')
    op.drop_table("metrics")
