<script setup lang="ts">
import { RouterView } from "vue-router";
</script>

<template lang="pug">
#app
  RouterView
</template>

<style lang="scss">
* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family:
    -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, "Helvetica Neue", Arial, sans-serif;
  background: #f1f5f9;
  color: #0f172a;
}

#app {
  min-height: 100vh;
}
</style>
