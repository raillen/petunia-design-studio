# Example: Limiting Concurrency with asyncio.Semaphore

When using asyncio to make many external network requests, it is easy to overwhelm the target server or run out of local file descriptors. Use an `asyncio.Semaphore` to limit concurrency.

## Code Example

```python
import asyncio
import aiohttp

async def fetch_url(session: aiohttp.ClientSession, url: str, semaphore: asyncio.Semaphore) -> str:
    # The semaphore limits the number of concurrent executions block here
    async with semaphore:
        print(f"Fetching {url}")
        async with session.get(url) as response:
            return await response.text()

async def main():
    urls = [f"https://example.com/page{i}" for i in range(100)]
    
    # Limit to 5 concurrent requests
    semaphore = asyncio.Semaphore(5)
    
    async with aiohttp.ClientSession() as session:
        tasks = [fetch_url(session, url, semaphore) for url in urls]
        
        # asyncio.gather runs all tasks concurrently, but the semaphore
        # ensures only 5 are ever actively executing the block inside `async with semaphore`
        results = await asyncio.gather(*tasks)
        print(f"Fetched {len(results)} pages.")

if __name__ == "__main__":
    asyncio.run(main())
```
