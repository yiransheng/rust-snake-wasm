import styles from './styles.css';

const js = import("./snake_wasm");

js.then(js => {
  js.main();
});
