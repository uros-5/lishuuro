<template>Logged</template>

<script setup lang="ts">
import router from '@/router'
import { onMounted } from 'vue'

onMounted(() => {
  router.push('/')
})
</script>
