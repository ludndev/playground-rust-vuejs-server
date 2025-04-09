import http from 'k6/http';
import { sleep } from 'k6';

export const options = {
  vus: 10,           // 10 virtual users (simultaneous connections)
  duration: '60s',   // Test duration: 60 seconds
};

export default function () {
  // Make HTTP request to your server
  http.get('http://127.0.0.1:8080');
  
  // Optional: add some sleep time between requests (e.g., 1 second)
  sleep(1);
}
