<script lang="ts">
  interface Props {
    size?: number;
  }
  let { size = 24 }: Props = $props();
</script>

<svg
  xmlns="http://www.w3.org/2000/svg"
  viewBox="0 0 512 512"
  width={size}
  height={size}
  aria-hidden="true"
>
  <rect width="512" height="512" rx="108" fill="#0098ff"/>
  <path d="M256 140 L370 206 L256 272 L142 206Z" fill="#fff" fill-opacity="0.4"/>
  <path d="M142 206 L256 272 L256 404 L142 338Z" fill="#fff" fill-opacity="0.95"/>
  <path d="M256 272 L370 206 L370 338 L256 404Z" fill="#fff" fill-opacity="0.65"/>
  <path d="M170 288 L235 326 L235 392 L170 354Z" fill="#005a9e"/>
</svg>
