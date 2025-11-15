// Example usage of the API client
import { ApiClient, ApiError } from './api-client';

async function main() {
  const client = new ApiClient('http://localhost:8000');

  try {
    // Check server health
    console.log('Checking server health...');
    const health = await client.health();
    console.log('Server status:', health);

    // Create users
    console.log('\nCreating users...');
    const alice = await client.users.create({
      name: 'Alice',
      email: 'alice@example.com',
    });
    console.log('Created user:', alice);

    const bob = await client.users.create({
      name: 'Bob',
      email: 'bob@example.com',
    });
    console.log('Created user:', bob);

    const charlie = await client.users.create({
      name: 'Charlie',
      email: 'charlie@example.com',
    });
    console.log('Created user:', charlie);

    // List all users
    console.log('\nListing all users...');
    const allUsers = await client.users.list();
    console.log(`Found ${allUsers.length} users:`, allUsers);

    // List with pagination
    console.log('\nListing with pagination (limit: 2)...');
    const page1 = await client.users.list({ limit: 2, offset: 0 });
    console.log('Page 1:', page1);

    const page2 = await client.users.list({ limit: 2, offset: 2 });
    console.log('Page 2:', page2);

    // Get single user
    console.log('\nGetting single user...');
    const user = await client.users.get(alice.id);
    console.log('Retrieved user:', user);

    // Update user
    console.log('\nUpdating user...');
    const updated = await client.users.update(alice.id, {
      name: 'Alice Smith',
      email: 'alice.smith@example.com',
    });
    console.log('Updated user:', updated);

    // Delete user
    console.log('\nDeleting user...');
    await client.users.delete(bob.id);
    console.log('User deleted successfully');

    // List remaining users
    console.log('\nListing remaining users...');
    const remaining = await client.users.list();
    console.log(`Remaining users (${remaining.length}):`, remaining);

    // Try to create user with invalid email
    console.log('\nTrying to create user with invalid email...');
    try {
      await client.users.create({
        name: 'Invalid',
        email: 'not-an-email',
      });
    } catch (error) {
      if (error instanceof ApiError) {
        console.log('Expected error:', error.message);
        console.log('Error details:', error.details);
      } else {
        throw error;
      }
    }

    // Try to get non-existent user
    console.log('\nTrying to get non-existent user...');
    try {
      await client.users.get('00000000-0000-0000-0000-000000000000');
    } catch (error) {
      if (error instanceof ApiError) {
        console.log('Expected error:', error.message);
      } else {
        throw error;
      }
    }

    console.log('\nAll examples completed successfully!');
  } catch (error) {
    if (error instanceof ApiError) {
      console.error('API Error:', error.message);
      console.error('Status:', error.status);
      console.error('Details:', error.details);
    } else {
      console.error('Unexpected error:', error);
    }
    process.exit(1);
  }
}

// Run the example
main().catch(console.error);
