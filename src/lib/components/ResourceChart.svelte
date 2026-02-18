<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { Chart, registerables } from "chart.js";

  Chart.register(...registerables);

  interface Props {
    data: number[];
    color: string;
    podId: string;
    metric: string;
  }

  let { data, color, podId, metric }: Props = $props();

  let canvas: HTMLCanvasElement;
  let chart: Chart | null = null;

  function createChart() {
    if (!canvas) return;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    chart = new Chart(ctx, {
      type: "line",
      data: {
        labels: data.map((_, i) => i.toString()),
        datasets: [
          {
            data: data,
            borderColor: color,
            backgroundColor: color + "1a",
            borderWidth: 1.5,
            fill: true,
            tension: 0.3,
            pointRadius: 0,
            pointHitRadius: 0,
          },
        ],
      },
      options: {
        responsive: false,
        maintainAspectRatio: false,
        animation: false,
        plugins: {
          legend: { display: false },
          tooltip: { enabled: false },
        },
        scales: {
          x: { display: false },
          y: { display: false, min: 0 },
        },
        elements: {
          line: { borderCapStyle: "round" },
        },
      },
    });
  }

  function updateChart() {
    if (!chart) return;
    chart.data.labels = data.map((_, i) => i.toString());
    chart.data.datasets[0].data = data;
    chart.update("none");
  }

  onMount(() => {
    createChart();
  });

  onDestroy(() => {
    chart?.destroy();
  });

  $effect(() => {
    // Re-run when data changes
    data;
    updateChart();
  });
</script>

<canvas
  bind:this={canvas}
  width="120"
  height="32"
  data-testid="pod-{metric}-{podId}"
  aria-label="{metric} chart for pod {podId}"
  style="width: 120px; height: 32px;"
></canvas>
