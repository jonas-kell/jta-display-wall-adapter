using Microsoft.AspNetCore.Builder;
using Microsoft.AspNetCore.Http;
using System.Formats.Nrbf;
using System.IO;
using System.Text.Json;
using System.Collections;
using System.Reflection;

var builder = WebApplication.CreateBuilder(args);

// Listen on all interfaces
builder.WebHost.UseUrls("http://0.0.0.0:8080");

var app = builder.Build();

// Recursive helper to convert NRBF decoded object into JSON-safe structure
object? ConvertNrbfToSafeObject(object? node, HashSet<object>? visited = null)
{
    if (node == null) return null;

    // Initialize visited set
    visited ??= new HashSet<object>(ReferenceEqualityComparer.Instance);

    // Avoid cycles
    if (!node.GetType().IsValueType) // only track reference types
    {
        if (visited.Contains(node))
            return "<circular reference>";
        visited.Add(node);
    }

    // Primitive types
    if (node is string or bool or byte or sbyte or short or ushort or int or uint or long or ulong or float or double or decimal)
        return node;

    // IDictionary
    if (node is IDictionary dict)
    {
        var result = new Dictionary<string, object?>();
        foreach (DictionaryEntry entry in dict)
        {
            var key = entry.Key?.ToString() ?? "null";
            result[key] = ConvertNrbfToSafeObject(entry.Value, visited);
        }
        return result;
    }

    // IEnumerable (arrays, lists)
    if (node is IEnumerable enumerable && node is not string)
    {
        var list = new List<object?>();
        foreach (var item in enumerable)
        {
            list.Add(ConvertNrbfToSafeObject(item, visited));
        }
        return list;
    }

    // Fallback: reflection
    var type = node.GetType();
    var dictObj = new Dictionary<string, object?> { ["__type"] = type.FullName };

    // Fields
    foreach (var field in type.GetFields(BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance))
    {
        try
        {
            dictObj[field.Name] = ConvertNrbfToSafeObject(field.GetValue(node), visited);
        }
        catch
        {
            dictObj[field.Name] = $"<unreadable field: {field.Name}>";
        }
    }

    // Properties
    foreach (var prop in type.GetProperties(BindingFlags.Public | BindingFlags.NonPublic | BindingFlags.Instance))
    {
        if (prop.GetIndexParameters().Length > 0) continue;
        try
        {
            dictObj[prop.Name] = ConvertNrbfToSafeObject(prop.GetValue(node), visited);
        }
        catch
        {
            dictObj[prop.Name] = $"<unreadable property: {prop.Name}>";
        }
    }

    return dictObj;
}

// POST /deserialize endpoint
app.MapPost("/deserialize", async (HttpRequest request) =>
{
    using var ms = new MemoryStream();
    if (request.ContentLength > 0)
        await request.Body.CopyToAsync(ms);

    if (ms.Length == 0)
        return Results.Problem(detail: "Empty request body", statusCode: 400);

    ms.Position = 0;

    try
    {
        // Decode NRBF payload safely with surrogate graph option
        var root = NrbfDecoder.Decode(ms);

        var safeObject = ConvertNrbfToSafeObject(root);

        var json = JsonSerializer.Serialize(safeObject, new JsonSerializerOptions
        {
            WriteIndented = true
        });

        return Results.Text(json, "application/json");
    }
    catch (Exception ex)
    {
        Console.Error.WriteLine($"NRBF decoding failed: {ex}");
        return Results.Problem(detail: ex.ToString(), statusCode: 400);
    }
});

// Health check endpoint
app.MapGet("/health", () => Results.Ok("Service is running"));

app.Run();
