import os
import glob
import psycopg2
from pathlib import Path
from dotenv import load_dotenv


def create_migration_tracking_table(cursor):
    """Create a table to track which migrations have been applied."""
    cursor.execute("""
        CREATE TABLE IF NOT EXISTS migration_history (
            id SERIAL PRIMARY KEY,
            filename VARCHAR(255) UNIQUE NOT NULL,
            applied_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        );
    """)


def check_table_exists(cursor, table_name):
    """Check if a table exists in the database."""
    cursor.execute("""
        SELECT EXISTS (
            SELECT FROM information_schema.tables 
            WHERE table_schema = 'public' 
            AND table_name = %s
        );
    """, (table_name,))
    exists = cursor.fetchone()[0]
    print(f"Table '{table_name}' exists: {exists}")
    return exists


def detect_applied_migrations(cursor):
    """Detect which migrations have likely been applied based on existing tables."""
    print("Detecting applied migrations from existing tables...")
    applied = set()
    
    # Check for tables from 001_initial.sql
    if check_table_exists(cursor, 'together_attestations'):
        applied.add('001_initial.sql')
        print("Detected 001_initial.sql already applied (together_attestations table exists)")
    
    # Check for tables from 002_pending_connections.sql  
    if check_table_exists(cursor, 'users'):
        applied.add('002_pending_connections.sql')
        print("Detected 002_pending_connections.sql already applied (users table exists)")
    
    print(f"Detection complete. Applied migrations: {applied}")
    return applied


def get_applied_migrations(cursor):
    """Get list of already applied migrations."""
    try:
        cursor.execute("SELECT filename FROM migration_history ORDER BY filename")
        applied_from_history = {row[0] for row in cursor.fetchall()}
        print(f"Migrations from history table: {applied_from_history}")
        
        # If history is empty but tables exist, detect from tables
        if not applied_from_history:
            print("Migration history is empty, detecting from existing tables...")
            return detect_applied_migrations(cursor)
        
        return applied_from_history
    except psycopg2.errors.UndefinedTable:
        # Migration history table doesn't exist yet, detect from existing tables
        print("Migration history table doesn't exist, detecting from existing tables...")
        return detect_applied_migrations(cursor)


def apply_migration(cursor, migration_file, content):
    """Apply a single migration and record it."""
    print(f"Applying migration: {migration_file}")
    
    # Execute the migration SQL
    cursor.execute(content)
    
    # Record that this migration was applied
    cursor.execute(
        "INSERT INTO migration_history (filename) VALUES (%s) ON CONFLICT (filename) DO NOTHING",
        (migration_file,)
    )


def main():
    # Load environment variables
    load_dotenv()
    
    database_url = os.getenv('DATABASE_PUBLIC_URL')
    # database_url = os.getenv('DATABASE_URL')
    if not database_url:
        print("Error: DATABASE_PUBLIC_URL environment variable not set")
        return 1
    
    # Get migration files
    migrations_dir = Path(__file__).parent / "../backend/migrations"
    migration_files = sorted(glob.glob(str(migrations_dir / "*.sql")))
    
    if not migration_files:
        print("No migration files found")
        return 0
    
    print(f"Found {len(migration_files)} migration files")
    
    try:
        # Connect to database
        conn = psycopg2.connect(database_url)
        conn.autocommit = False
        
        with conn.cursor() as cursor:
            # Create migration tracking table
            print("Creating migration tracking table...")
            create_migration_tracking_table(cursor)
            conn.commit()
            
            # Get already applied migrations
            print("Checking for already applied migrations...")
            applied_migrations = get_applied_migrations(cursor)
            print(f"Found {len(applied_migrations)} already applied migrations: {applied_migrations}")
            
            # If we detected migrations from existing tables, record them in migration_history
            if applied_migrations:
                print("Recording detected migrations in migration_history...")
                for migration_file in applied_migrations:
                    cursor.execute(
                        "INSERT INTO migration_history (filename) VALUES (%s) ON CONFLICT (filename) DO NOTHING",
                        (migration_file,)
                    )
                conn.commit()
                print("Recorded detected migrations")
            
            # Apply pending migrations
            for migration_path in migration_files:
                migration_file = os.path.basename(migration_path)
                
                if migration_file in applied_migrations:
                    print(f"Skipping already applied migration: {migration_file}")
                    continue
                
                # Read migration content
                with open(migration_path, 'r') as f:
                    content = f.read()
                
                try:
                    apply_migration(cursor, migration_file, content)
                    conn.commit()
                    print(f"Successfully applied: {migration_file}")
                except Exception as e:
                    conn.rollback()
                    print(f"Error applying migration {migration_file}: {e}")
                    return 1
        
        conn.close()
        print("All migrations applied successfully!")
        return 0
        
    except Exception as e:
        print(f"Database connection error: {e}")
        return 1


if __name__ == "__main__":
    exit(main())
