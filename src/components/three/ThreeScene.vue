<script setup lang="ts">
import { useThreeScene } from '../../composables/useThreeScene'

const { threeContainer } = useThreeScene()
</script>

<template>
  <div ref="threeContainer" class="absolute inset-0 z-0"></div>
</template> 