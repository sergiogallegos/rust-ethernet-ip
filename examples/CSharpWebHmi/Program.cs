using CSharpWebHmi.Services;

var builder = WebApplication.CreateBuilder(args);

builder.Services.AddSingleton<PlcDashboardService>();
builder.Services.AddCors(options =>
{
    options.AddPolicy("FrontendDevelopment", policy =>
        policy.WithOrigins("http://127.0.0.1:5173", "http://localhost:5173")
            .AllowAnyHeader()
            .AllowAnyMethod());
});

var app = builder.Build();

app.UseCors("FrontendDevelopment");
app.Use(async (context, next) =>
{
    if (context.Request.Path == "/")
    {
        context.Response.Headers.CacheControl = "no-store, no-cache, must-revalidate";
        context.Response.Headers.Pragma = "no-cache";
        context.Response.Headers.Expires = "0";
    }
    await next(context);
});
app.UseDefaultFiles();
app.UseStaticFiles(new StaticFileOptions
{
    OnPrepareResponse = context =>
    {
        if (context.Context.Request.Path == "/"
            || string.Equals(context.File.Name, "index.html", StringComparison.OrdinalIgnoreCase))
        {
            context.Context.Response.Headers.CacheControl = "no-store, no-cache, must-revalidate";
            context.Context.Response.Headers.Pragma = "no-cache";
            context.Context.Response.Headers.Expires = "0";
        }
    },
});

app.MapGet("/api/dashboard", async (PlcDashboardService dashboard, CancellationToken cancellationToken) =>
    Results.Ok(await dashboard.GetSnapshotAsync(cancellationToken)));

app.MapPost("/api/dashboard/pulse", async (PlcDashboardService dashboard, CancellationToken cancellationToken) =>
{
    var result = await dashboard.PulseTestBoolAsync(cancellationToken);
    return result.Success ? Results.Ok(result) : Results.BadRequest(result);
});

app.MapGet("/api/health", () => Results.Ok(new { status = "ok" }));
app.MapFallbackToFile("index.html");

app.Run();
