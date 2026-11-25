window.onload = () => {
  window.SwaggerUIBundle({
    url: '/api-docs/openapi.json',
    dom_id: '#swagger-ui',
    withCredentials: true,
    persistAuthorization: false,
  });
};
